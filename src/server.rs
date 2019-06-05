use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Token, Ready, PollOpt, Events};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use std::str;
use std::io;
use std::io::prelude::*;

const SERVER: Token = Token(0);
const READ_BUFFER_SIZE: usize = 4096;

struct Connection {
    user: String,
    stream: TcpStream,
}

pub struct Server<C>
{
    listener: TcpListener,
    read_buffer: [u8; READ_BUFFER_SIZE],
    poll: Poll,
    events: Events,
    connections: HashMap<Token, Connection>,
    connections_accepted: usize,
    on_read: C,
}

impl<C> Server<C>
where C: FnMut(&str, SocketAddr, &[u8]) -> bool,
{
    pub fn new(addr: &SocketAddr, on_read: C) -> Server<C> {
        let listener = TcpListener::bind(addr).unwrap();
        let poll = Poll::new().unwrap();
        poll.register(&listener, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

        Server {
            listener,
            read_buffer: [0; READ_BUFFER_SIZE],
            poll,
            events: Events::with_capacity(1024),
            connections: HashMap::new(),
            connections_accepted: 0,
            on_read,
        }
    }

    pub fn listen(&mut self, timeout: Option<Duration>) {
        match self.poll.poll(&mut self.events, timeout) {
            Ok(_) => {
                for event in self.events.iter() {
                    match event.token() {
                        SERVER => {
                            let (stream, _) = self.listener.accept().unwrap();

                            self.connections_accepted += 1;
                            let token = Token(self.connections_accepted);

                            self.poll.register(&stream, token, Ready::readable(), PollOpt::edge()).unwrap();
                            self.connections.insert(token, Connection{ user: String::new(), stream });
                        },
                        token => {
                            let connection = self.connections.get_mut(&token).unwrap();
                            let size = connection.stream.read(&mut self.read_buffer).unwrap();
                            if connection.user.is_empty() {
                                connection.user = str::from_utf8(&self.read_buffer[0..size]).unwrap().to_string();
                            }
                            else {
                                let addr = connection.stream.local_addr().unwrap();
                                if !(self.on_read)(&connection.user, addr, &self.read_buffer[0..size]) {
                                    self.poll.deregister(&connection.stream).unwrap();
                                    self.connections.remove(&token);
                                }
                            }
                        },
                    }
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                io::ErrorKind::TimedOut => (),
                _ => Err(e).unwrap(),
            }
        }
    }

    pub fn get_listener_port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }
}
