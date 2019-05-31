use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use std::io;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: &SocketAddr) -> io::Result<Server> {
        Ok(Server {
            listener: TcpListener::bind(addr).unwrap(),
        })
    }

    pub fn listen(&self, _timeout: Duration) -> io::Result<()> {
        Ok(())
    }

    pub fn get_listener_port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }
}
