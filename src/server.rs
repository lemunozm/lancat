use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Token, Ready, PollOpt, Events};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use std::io;
use std::io::prelude::*;

const SERVER: Token = Token(0);
const READ_BUFFER_SIZE: usize = 4096;

struct Connection {
    user: String,
    stream: TcpStream,
}

pub struct Server<A, R>
{
    listener: TcpListener,
    read_buffer: [u8; READ_BUFFER_SIZE],
    poll: Poll,
    events: Events,
    connections: HashMap<Token, Connection>,
    connections_accepted: usize,
    on_accept: A,
    on_read: R,
}

impl<A, R> Server<A, R>
where A: Fn(&str) -> io::Result<bool>,
      R: Fn(&str, &[u8], usize) -> io::Result<()>,
{
    pub fn new(addr: &SocketAddr, on_accept: A, on_read: R) -> io::Result<Server<A, R>> {
        let listener = TcpListener::bind(addr)?;
        let poll = Poll::new()?;
        poll.register(&listener, SERVER, Ready::readable(), PollOpt::edge())?;

        Ok(Server {
            listener,
            read_buffer: [0; READ_BUFFER_SIZE],
            poll,
            events: Events::with_capacity(1024),
            connections: HashMap::new(),
            connections_accepted: 0,
            on_accept,
            on_read,
        })
    }

    pub fn listen(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        match self.poll.poll(&mut self.events, timeout) {
            Ok(_) => {
                for event in self.events.iter() {
                    match event.token() {
                        SERVER => {
                            let (mut stream, _address) = self.listener.accept()?;
                            let mut user = String::new();
                            stream.read_to_string(&mut user)?;

                            (self.on_accept)(&user)?;

                            self.connections_accepted += 1;
                            let token = Token(self.connections_accepted);
                            self.poll.register(&stream, token, Ready::readable(), PollOpt::edge())?;
                            self.connections.insert(token, Connection{ user, stream });
                        },
                        token => {
                            let connection = self.connections.get_mut(&token).unwrap();
                            let size = connection.stream.read(&mut self.read_buffer)?;

                            (self.on_read)(&connection.user, &self.read_buffer, size)?;
                        }
                    }
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                io::ErrorKind::TimedOut => (),
                _ => return Err(e),
            }
        }
        Ok(())
    }

    pub fn get_listener_port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }
}
