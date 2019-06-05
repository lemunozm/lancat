use serde::{Serialize, Deserialize};
use net2::UdpBuilder;

use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr, UdpSocket};
use std::time::Duration;
use std::io;
use std::str;

const READ_BUFFER_SIZE: usize = 256;
const DISCOVER_MAX: usize = 100;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct DiscoveryInfo {
    name: String,
    port: u16,
}

#[derive(PartialEq, Clone)]
pub struct EndpointInfo {
    pub name: String,
    pub addr: SocketAddr,
}

pub struct DiscoveryServer {
    discovery_addr: SocketAddrV4,
    serialized_info: Vec<u8>,
    socket: UdpSocket,
}

pub fn discover(discovery_addr: &SocketAddrV4) -> Vec<EndpointInfo> {
    let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let socket = UdpSocket::bind(&local_addr).unwrap();
    socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
    socket.send_to(&[0], discovery_addr).unwrap();

    let mut buffer = [0; READ_BUFFER_SIZE];
    let mut endpoints = Vec::new();

    for _ in 0..DISCOVER_MAX {
        match socket.recv_from(&mut buffer) {
            Ok((size, remote_addr)) => {
                let remote_info: DiscoveryInfo = bincode::deserialize(&buffer[0..size]).unwrap();
                let endpoint = EndpointInfo {
                    name: remote_info.name,
                    addr: SocketAddr::new(remote_addr.ip(), remote_info.port),
                };

                endpoints.push(endpoint);
            },
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => break,
                io::ErrorKind::TimedOut => break,
                e => Err(e).unwrap(),
            }
        };
    }
    endpoints
}

impl DiscoveryServer {
    pub fn new(discovery_addr: &SocketAddrV4, service_name: &String, service_port: u16) -> DiscoveryServer {
        let info = DiscoveryInfo {
            name: service_name.clone(),
            port: service_port,
        };

        let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, discovery_addr.port());
        let socket = UdpBuilder::new_v4().unwrap().reuse_address(true).unwrap().bind(local_addr).unwrap();
        socket.join_multicast_v4(&discovery_addr.ip(), &Ipv4Addr::UNSPECIFIED).unwrap();

        DiscoveryServer {
            discovery_addr: discovery_addr.clone(),
            serialized_info: bincode::serialize(&info).unwrap(),
            socket: socket,
        }
    }

    pub fn listen(&self, timeout: Option<Duration>) {
        let mut buffer = [0; 0]; //CHECK: the behaviour in windows
        self.socket.set_read_timeout(timeout).unwrap();
        match self.socket.recv_from(&mut buffer) {
            Ok((_, remote_addr)) => {
                loop {
                    match self.socket.send_to(&self.serialized_info, remote_addr) {
                        Ok(_) => break (),
                        Err(e) => match e.kind() {
                            io::ErrorKind::PermissionDenied => (),
                            _ => Err(e).unwrap(),
                        }
                    };
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                io::ErrorKind::TimedOut => (),
                _ => Err(e).unwrap(),
            }
        }
    }
}

impl Drop for DiscoveryServer {
    fn drop(&mut self) {
        self.socket.leave_multicast_v4(&self.discovery_addr.ip(), &Ipv4Addr::UNSPECIFIED).unwrap();
    }
}

