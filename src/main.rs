extern crate whoami;

mod discovery;

fn main() {
    let discovery_server = discovery::Server::new(2000, &whoami::username());
    discovery_server.run(2001).unwrap();
}
