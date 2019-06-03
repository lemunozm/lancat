#[macro_use]
extern crate clap;
extern crate whoami;
extern crate bincode;
extern crate serde;
extern crate net2;

mod server;
mod discovery;

use server::Server;
use discovery::{DiscoveryServer, EndpointInfo};

use std::io;
use std::io::Write;
use std::thread;
use std::time::Duration;
use std::net::TcpStream;

const SEARCH: &str = "search";
const LISTEN: &str = "listen";
const USERS: &str = "users";
const DISCOVERY_PORT: &str = "discovery-port";
const DISCOVERY_IP: &str = "discovery-ip";
const SERVICE_PORT: &str = "service-port";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let matches = clap::App::new("lancat")
        .version(&crate_version!()[..])
        .version_short("v")
        .arg(clap::Arg::with_name(SEARCH)
            .long(SEARCH)
            .short("s")
            .help("Only list the users in the LAN")
            .conflicts_with(LISTEN)
        )
        .arg(clap::Arg::with_name(LISTEN)
            .long(LISTEN)
            .short("l")
            .help("Listen mode")
            .conflicts_with(SEARCH)
        )
        .arg(clap::Arg::with_name(USERS)
            .long(USERS)
            .short("u")
            .value_name("users")
            .help("User list to take into account for the communication")
            .default_value("")
            .hide_default_value(true)
        )
        .arg(clap::Arg::with_name(SERVICE_PORT)
            .long(SERVICE_PORT)
            .short("c")
            .value_name("number")
            .default_value("0")
            .help("Port used for cat communication")
        )
        .arg(clap::Arg::with_name(DISCOVERY_PORT)
            .long(DISCOVERY_PORT)
            .short("p")
            .value_name("number")
            .default_value("2002")
            .help("Port used for discover 'lancat's listening in the LAN")
        )
        .arg(clap::Arg::with_name(DISCOVERY_IP)
            .long(DISCOVERY_IP)
            .short("d")
            .value_name("ip")
            .default_value("239.255.0.1")
            .help("Multicast ip used for discovery")
        )
        .get_matches_from(args);


    let service_port = value_t!(matches, SERVICE_PORT, String).unwrap();
    let service_addr = format!("0.0.0.0:{}", service_port).parse().unwrap();

    let discovery_ip = value_t!(matches, DISCOVERY_IP, String).unwrap();
    let discovery_port = value_t!(matches, DISCOVERY_PORT, String).unwrap();
    let discovery_addr = format!("{}:{}", discovery_ip, discovery_port).parse().unwrap();

    let users = values_t!(matches, USERS, String).unwrap();

    if matches.is_present(SEARCH) {
        let remotes = discovery::discover(&discovery_addr).unwrap();
        let remotes = filter_users(&remotes, &users);
        for remote in remotes.iter() {
            println!("Found '{}' at: {}", remote.name, remote.addr);
        }
    }
    else if matches.is_present(LISTEN) {
        let on_accept = |_user: &str| -> io::Result<bool> {
            Ok(true)
        };

        let on_data = |_user: &str, data: &[u8], _size: usize| -> io::Result<()> {
            println!("data len {}", data.len());
            io::stdout().write(data)?;
            Ok(())
        };

        let mut server = Server::new(&service_addr, on_accept, on_data).unwrap();
        let listener_port = server.get_listener_port();
        let server_join = thread::spawn(move || {
            loop {
                server.listen(Some(Duration::from_millis(100))).unwrap();
            }
        });

        let discovery_server = DiscoveryServer::new(&discovery_addr, &whoami::username(), listener_port).unwrap();
        let discovery_join = thread::spawn(move || {
            loop {
                discovery_server.listen(Some(Duration::from_millis(100))).unwrap();
            }
        });

        discovery_join.join().unwrap();
        server_join.join().unwrap();
    }
    else {
        let remotes = discovery::discover(&discovery_addr).unwrap();
        let remotes = filter_users(&remotes, &users);
        let mut connections = vec![];
        for remote in remotes.iter() {
            let mut connection = TcpStream::connect(remote.addr).unwrap();
            connection.write(&whoami::username().as_bytes()).unwrap();
            connections.push(connection);
        }

        let mut input = String::new();
        loop {
            io::stdin().read_line(&mut input).unwrap();
            for mut connection in &connections {
                connection.write(&input.as_bytes()).unwrap();
            }
        }
    }
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
