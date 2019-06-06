extern crate bincode;
extern crate serde;
extern crate net2;
extern crate crossbeam;

mod server;
mod discovery;

use crossbeam::thread;

use server::Server;
use discovery::{DiscoveryServer, EndpointInfo};

use std::io::Write;
use std::io::Read;
use std::time::Duration;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};

const READ_BUFFER_SIZE: usize = 1024;

pub fn search(discovery_addr: &SocketAddrV4, users: Option<&Vec<String>>) {
    let remotes = find_remotes(discovery_addr, users);

    for remote in remotes.iter() {
        println!("Found '{}' at: {}", remote.name, remote.addr);
    }
}

pub fn speak<R>(discovery_addr: &SocketAddrV4, users: Option<&Vec<String>>, user_name: &str, mut input: R)
where R: Read + 'static {
    let remotes = find_remotes(discovery_addr, users);

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
        for mut connection in &connections {
            connection.write(&input_buffer[0..size]).unwrap();
        }
    }
}

pub fn listen<W>(
        discovery_addr: &SocketAddrV4,
        users: Option<&Vec<String>>,
        user_name: &str,
        service_addr: &SocketAddr,
        mut output: W)
where
    W: Write + Send + 'static
{
    let mut last_print_user = String::new();
    let on_data = move |user: &str, remote: SocketAddr, data: &[u8]| -> bool {
        if let Some(users) = users {
            return !users.iter().any(|u| u == user);
        }

        if last_print_user != user {
            println!("============ {} - {} ============", user, remote); //TODO: terminal size
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

fn find_remotes(discovery_addr: &SocketAddrV4, users: Option<&Vec<String>>) -> Vec<EndpointInfo> {
    let remotes = discovery::discover(&discovery_addr);
    match users {
        Some(users) => remotes.into_iter().filter(|r| users.iter().any(|u| *u == r.name)).collect(),
        None => remotes,
    }
}
