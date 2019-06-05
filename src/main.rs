#[macro_use]
extern crate clap;
extern crate whoami;
extern crate lancat;

use std::io;

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
        lancat::search(discovery_addr, users);
    }
    else if matches.is_present(LISTEN) {
        lancat::listen(user_name, users, discovery_addr, service_addr, io::stdout());
    }
    else {
        lancat::speak(user_name, users, discovery_addr, io::stdin());
    }
}

