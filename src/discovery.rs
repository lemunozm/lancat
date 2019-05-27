use std::net::{SocketAddr, IpAddr, Ipv4Addr, UdpSocket};
use std::io;
use std::str;

pub struct EndpointInfo {
    address: SocketAddr,
    name: String,
}

pub struct Server {
    local_info: EndpointInfo,
}

pub fn discover() -> Vec<EndpointInfo> {
    Vec::new()
}

impl Server {
    pub fn new(service_port: u16, username: &String) -> Server {
        let local_info = EndpointInfo {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), service_port),
            name: username.to_string(),
        };

        Server { local_info: local_info }
    }

    pub fn run(&self, listen_port: u16) -> io::Result<()> {
        let local_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), listen_port);
        let mut socket = UdpSocket::bind(&local_address)?;

        let mut buffer = [0; 128];
        let (size, remote_address) = socket.recv_from(&mut buffer)?;
        println!("Received message: {}, bytes: {}, from: {}", &str::from_utf8(&buffer).unwrap(), size, remote_address);
        Ok(())
    }
}

