#[macro_use]
extern crate clap;
extern crate whoami;
extern crate bincode;
extern crate serde;

mod discovery;

use discovery::DiscoveryServer;
use std::net::TcpListener;

const SEARCH: &str = "search";
const DISCOVERY_PORT: &str = "discovery-port";
const DISCOVERY_IP: &str = "discovery_ip";
const SERVICE_PORT: &str = "service_port";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let matches = clap::App::new("lancat")
        .version(&crate_version!()[..])
        .version_short("v")
        .arg(clap::Arg::with_name(SEARCH)
            .long(SEARCH)
            .short("s")
            .help("only list the users in the lan")
        )
        .arg(clap::Arg::with_name(SERVICE_PORT)
            .long(SERVICE_PORT)
            .short("c")
            .value_name("number")
            .default_value("0")
            .help("port used for cat communication")
        )
        .arg(clap::Arg::with_name(DISCOVERY_PORT)
            .long(DISCOVERY_PORT)
            .short("p")
            .value_name("number")
            .default_value("2002")
            .help("port used for discover 'lancat's in the lan")
        )
        .arg(clap::Arg::with_name(DISCOVERY_IP)
            .long(DISCOVERY_IP)
            .value_name("ip")
            .default_value("239.255.0.1")
            .help("multicast ip used for discovery")
        )
        .get_matches_from(args);


    let service_port = value_t!(matches, SERVICE_PORT, String).unwrap();
    let service_addr = format!("0.0.0.0:{}", service_port);

    let discovery_ip = value_t!(matches, DISCOVERY_IP, String).unwrap();
    let discovery_port = value_t!(matches, DISCOVERY_PORT, String).unwrap();
    let discovery_addr = format!("{}:{}", discovery_ip, discovery_port).parse().unwrap();

    let listener = TcpListener::bind(service_addr).unwrap();
    let listener_port = listener.local_addr().unwrap().port();

    let discovery_server = DiscoveryServer::new(&discovery_addr, &whoami::username(), listener_port);

    if matches.is_present(SEARCH) {
        println!("Searching at {} ...", &discovery_addr);

        let discovered = discovery_server.discover().unwrap();
        for remote in discovered.iter() {
            println!("Found remote at: {} - {}", remote.name, remote.addr);
        }
    }
    else {
        println!("Listening...");
        discovery_server.run().unwrap();
    }
}
