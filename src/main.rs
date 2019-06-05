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
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};

const SEARCH: &str = "search";
const LISTEN: &str = "listen";
const USERS: &str = "users";
const NAME: &str = "name";
const DISCOVERY_PORT: &str = "discovery-port";
const DISCOVERY_IP: &str = "discovery-ip";
const SERVICE_PORT: &str = "service-port";

fn main() {
    let user = whoami::username();
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
        )
        .arg(clap::Arg::with_name(NAME)
            .long(NAME)
            .short("n")
            .value_name("user name")
            .default_value(&user)
            .help("User name identification in the LAN")
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

    let user_name = value_t!(matches, NAME, String).unwrap();

    let users =
    if matches.is_present(USERS) {
        values_t!(matches, USERS, String).unwrap()
    }
    else {
        vec![]
    };

    if matches.is_present(SEARCH) {
        search(discovery_addr, users);
    }
    else if matches.is_present(LISTEN) {
        listen(user_name, users, discovery_addr, service_addr, io::stdout());
    }
    else {
        speak(user_name, users, discovery_addr, io::stdin());
    }
}

fn search(discovery_addr: SocketAddrV4, users: Vec<String>) {
    let remotes = discovery::discover(&discovery_addr);
    let remotes = filter_users(&remotes, &users);

    for remote in remotes.iter() {
        println!("Found '{}' at: {}", remote.name, remote.addr);
    }
}

fn speak<R: Read + 'static>(user_name: String, users: Vec<String>, discovery_addr: SocketAddrV4, mut input: R) {
    let remotes = discovery::discover(&discovery_addr);
    let remotes = filter_users(&remotes, &users);

    let mut connections = vec![];
    for remote in remotes.iter() {
        let mut connection = TcpStream::connect(remote.addr).unwrap();
        connection.write(&user_name.as_bytes()).unwrap();
        connections.push(connection);
    }

    loop {
        let mut input_buffer = [0; 4096];
        let size = input.read(&mut input_buffer).unwrap();
        for mut connection in &connections {
            connection.write(&input_buffer[0..size]).unwrap();
        }
    }
}

fn listen<W: Write + Send + 'static>(
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
