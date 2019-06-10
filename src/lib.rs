extern crate bincode;
extern crate serde;
extern crate net2;
extern crate crossbeam;
extern crate term_size;

mod server;
mod discovery;

use crossbeam::thread;

use server::Server;
use discovery::DiscoveryServer;

use std::io;
use std::io::Write;
use std::io::Read;
use std::time::Duration;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};

const READ_BUFFER_SIZE: usize = 1024;

pub fn search(discovery_addr: &SocketAddrV4) {
    let remotes = discovery::discover(&discovery_addr);
    for remote in remotes.iter() {
        println!("Found '{}' at: {}", remote.name, remote.addr);
    }
}

pub fn talk<R>(discovery_addr: &SocketAddrV4, users: Option<&Vec<String>>, user_name: &str, mut input: R)
where R: Read + 'static {
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
           break
        }

        connections.retain(|mut connection|{
            match connection.write(&input_buffer[0..size]) {
                Ok(_) => true,
                Err(e) => match e.kind() {
                    io::ErrorKind::BrokenPipe => false,
                    _ => Err(e).unwrap(),
                },
            }
        });
    }
}

pub fn listen<W>(
        discovery_addr: &SocketAddrV4,
        users: Option<&Vec<String>>,
        user_name: &str,
        service_addr: &SocketAddr,
        verbose: bool,
        mut output: W)
where
    W: Write + Send + 'static
{
    let mut last_print_user = String::new();
    let on_data = move |user: &str, remote: &SocketAddr, data: &[u8]| -> bool {
        if let Some(users) = users {
            return !users.iter().any(|u| u == user);
        }

        if verbose && last_print_user != user {
            print_user_division(user, &remote);
            last_print_user = String::from(user);
        }

        output.write(data).unwrap();
        true
    };

    let mut server = Server::new(&service_addr, on_data);
    let discovery_server = DiscoveryServer::new(&discovery_addr, &user_name, server.get_listener_port());
    thread::scope(|s| {
        s.spawn(|_| {
            loop {
                server.listen(Some(Duration::from_millis(100)));
            }
        });

        s.spawn(|_| {
            loop {
                discovery_server.listen(Some(Duration::from_millis(100)));
            }
        });
    }).unwrap();
}

fn print_user_division(name: &str, remote: &SocketAddr) {
    let term_width = term_size::dimensions().unwrap().0;
    let info = format!(" {} - {} ", name, remote);
    let margin_width = (term_width - info.len()) / 2;
    let margin = String::from_utf8(vec![b'='; margin_width]).unwrap();
    let extra_digit = term_width > margin_width * 2 + info.len();
    println!("{}{}{}{}", margin, info, margin, if extra_digit {"="} else {""});
}
