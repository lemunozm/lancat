extern crate bincode;
extern crate serde;
extern crate net2;
extern crate crossbeam;
extern crate term_size;

mod server;
pub mod discovery;

use crossbeam::thread;

use server::Server;
use discovery::DiscoveryServer;

use std::io;
use std::io::Write;
use std::io::Read;
use std::time::Duration;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

const READ_BUFFER_SIZE: usize = 1024;

pub fn talk<R>(
        discovery_addr: &SocketAddrV4,
        users: Option<&Vec<String>>,
        user_name: &str,
        mut input: R) -> bool
where
    R: Read + 'static
{
    let remotes = discovery::discover(&discovery_addr);
    let remotes = match users {
        Some(users) => remotes.into_iter().filter(|r| users.iter().any(|u| *u == r.name)).collect(),
        None => remotes,
    };

    let mut connections = vec![];
    for remote in remotes.iter() {
        let mut connection = TcpStream::connect(remote.addr).unwrap();
        let serialized_user_name = bincode::serialize(&user_name).unwrap();
        connection.write(&serialized_user_name).unwrap();
        connections.push(connection);
    }

    loop {
        let mut input_buffer = [0; READ_BUFFER_SIZE];
        let size = input.read(&mut input_buffer).unwrap();
        if size == 0 {
            break;
        }

        connections.retain(|mut connection|{
            match connection.write(&input_buffer[0..size]) {
                Ok(_) => true,
                Err(e) => match e.kind() {
                    io::ErrorKind::BrokenPipe => false,
                    io::ErrorKind::ConnectionReset => false,
                    _ => Err(e).unwrap(),
                },
            }
        });

        if connections.len() == 0 {
            return false;
        }
    }
    true
}

pub fn listen<W, C>(
        discovery_addr: &SocketAddrV4,
        users: Option<&Vec<String>>,
        user_name: &str,
        service_addr: &SocketAddr,
        once: bool,
        mut callback: C,
        mut output: W)
where
    C: FnMut(&str, &SocketAddr) + Send + Sync,
    W: Write + Send
{
    let mut last_user = String::new();
    let on_data = |user: &str, remote: &SocketAddr, data: &[u8]| -> bool {
        if let Some(users) = users {
            if !users.iter().any(|u| u == user) {
                return false;
            }
        }
        if last_user != user {
            callback(user, remote);
            last_user = String::from(user);
        }
        output.write(data).unwrap();
        true
    };
    let mut server = Server::new(&service_addr, once, on_data);
    let discovery_server = DiscoveryServer::new(&discovery_addr, &user_name, server.get_listener_port());

    let running = AtomicBool::new(true);
    thread::scope(|s| {
        s.spawn(|_| {
            while running.load(Ordering::Relaxed) {
                let alive = server.listen(Some(Duration::from_millis(100)));
                running.store(alive, Ordering::Relaxed);
            }
        });

        s.spawn(|_| {
            while running.load(Ordering::Relaxed) {
                discovery_server.listen(Some(Duration::from_millis(100)));
            }
        });
    }).unwrap();
}

