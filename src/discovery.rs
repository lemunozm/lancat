use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr, UdpSocket};
use std::time::Duration;
use std::io;
use std::str;
use serde::{Serialize, Deserialize};

const READ_BUFFER_SIZE: usize = 256;
const DISCOVER_MAX: usize = 100;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryInfo {
    name: String,
    port: u16,
}

pub struct EndpointInfo {
    pub name: String,
    pub addr: SocketAddr,
}

pub struct DiscoveryServer {
    discovery_addr: SocketAddrV4,
    serialized_info: Vec<u8>,
    local_info: DiscoveryInfo,
}

impl DiscoveryServer {
    pub fn new(discovery_addr: &SocketAddrV4, service_name: &String, service_port: u16) -> DiscoveryServer {
        let info = DiscoveryInfo {
            name: service_name.clone(),
            port: service_port,
        };

        DiscoveryServer {
            discovery_addr: discovery_addr.clone(),
            serialized_info: bincode::serialize(&info).unwrap(),
            local_info: info,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, self.discovery_addr.port());
        let socket = UdpSocket::bind(&local_addr)?; //TODO: reuseaddr
        socket.join_multicast_v4(&self.discovery_addr.ip(), &Ipv4Addr::UNSPECIFIED)?;

        loop {
            let mut buffer = [0; READ_BUFFER_SIZE];
            let (_, remote_addr) = socket.recv_from(&mut buffer)?; //TODO: timeout
            socket.send_to(&self.serialized_info, remote_addr)?;
        }

        //TODO: socket.leave_multicast_v4(&self.discovery_addr.ip(), &Ipv4Addr::UNSPECIFIED)?;
    }

    pub fn discover(&self) -> io::Result<Vec<EndpointInfo>> {
        let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let socket = UdpSocket::bind(&local_addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(50)))?;
        socket.send_to(&[0], self.discovery_addr)?; //TODO: test with 0 bytes sent.

        let mut buffer = [0; READ_BUFFER_SIZE];
        let mut endpoints = Vec::new();

        for _ in 0..DISCOVER_MAX {
            match socket.recv_from(&mut buffer) {
                Ok((size, remote_addr)) => {
                    let remote_info: DiscoveryInfo = bincode::deserialize(&buffer[0..size]).unwrap();
                    if remote_info != self.local_info {
                        let endpoint = EndpointInfo {
                            name: remote_info.name,
                            addr: SocketAddr::new(remote_addr.ip(), remote_info.port),
                        };

                        //check is no
                        endpoints.push(endpoint);
                    }
                },
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => break,
                    io::ErrorKind::TimedOut => break,
                    _ => Err(e)?,
                }
            };
        }
        Ok(endpoints)
    }
}

