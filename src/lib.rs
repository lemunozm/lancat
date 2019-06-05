extern crate bincode;
extern crate serde;
extern crate net2;

mod server;
mod discovery;

use server::Server;
use discovery::{DiscoveryServer, EndpointInfo};

use std::io::Write;
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};

const READ_BUFFER_SIZE: usize = 4096;

pub fn search(discovery_addr: SocketAddrV4, users: Vec<String>) {
    let remotes = discovery::discover(&discovery_addr);
    let remotes = filter_users(&remotes, &users);

    for remote in remotes.iter() {
        println!("Found '{}' at: {}", remote.name, remote.addr);
    }
}

pub fn speak<R: Read + 'static>(user_name: String, users: Vec<String>, discovery_addr: SocketAddrV4, mut input: R) {
    let remotes = discovery::discover(&discovery_addr);
    let remotes = filter_users(&remotes, &users);

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

pub fn listen<W: Write + Send + 'static>(
        user_name: String,
        users: Vec<String>,
        discovery_addr: SocketAddrV4,
        service_addr: SocketAddr,
        mut output: W) {

    let mut last_print_user = String::new();

    let on_data = move |user: &str, remote: SocketAddr, data: &[u8]| {
        if !users.is_empty() && !users.iter().any(|x| x == user) {
            return false;
        }

        if last_print_user != user {
            println!("============ {} - {} ============", user, remote); //TODO: terminal size
            last_print_user = String::from(user);
        }

        output.write(data).unwrap();
        true
    };

    let mut server = Server::new(&service_addr, on_data);
    let listener_port = server.get_listener_port();
    let server_join = thread::spawn(move || {
        loop {
            server.listen(Some(Duration::from_millis(100)));
        }
    });

    let discovery_server = DiscoveryServer::new(&discovery_addr, &user_name, listener_port);
    let discovery_join = thread::spawn(move || {
        loop {
            discovery_server.listen(Some(Duration::from_millis(100)));
        }
    });

    discovery_join.join().unwrap();
    server_join.join().unwrap();
}

fn filter_users(remotes: &Vec<EndpointInfo>, users: &Vec<String>) -> Vec<EndpointInfo> {
    if users.is_empty() {
        return remotes.clone();
    }

    let mut filtered = vec![];
    for remote in remotes {
        for user in users {
            if remote.name == *user {
                filtered.push(remote.clone());
            }
        }
    }
    filtered
}
