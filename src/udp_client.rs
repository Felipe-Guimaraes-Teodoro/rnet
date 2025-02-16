use std::{collections::HashMap, net::SocketAddr};
use tokio::net::UdpSocket;

use crate::{Packet, Packets, NET_BUFFER_SIZE};

// I'd tell you a UDP joke... but you might not get it

pub struct UdpClient {
    socket: UdpSocket,
    // our_addr: SocketAddr,
    server_addr: SocketAddr,
    buf: [u8; NET_BUFFER_SIZE],

    pub client_packets: Packets, // client's packets
    pub server_packets: Packets, // packets received from srever
}

impl UdpClient {
    pub async fn new(client_addr: &str, server_addr: &str) -> Self {
        let socket: UdpSocket = UdpSocket::bind(client_addr).await.unwrap();

        let buf = [0 as u8; NET_BUFFER_SIZE];
        
        Self {
            socket, 
            // our_addr: client_addr.parse::<SocketAddr>().unwrap(), 
            server_addr: server_addr.parse::<SocketAddr>().unwrap(), 
            buf, 
            client_packets: Packets {
                packets: HashMap::new(),
            },
            server_packets: Packets {
                packets: HashMap::new(),
            },
        }
    }
    
    pub fn update(&mut self){
        if let Ok(_) = self.socket.try_recv(&mut self.buf) {
            self.server_packets = Packets::deserialize(&self.buf);
        }
    }

    // todo: impl for try_send and send be separate, instead of doing this weird way
    /// Sends a message to the server in the form of a neat package
    /// ```rs
    /// let message = Packet::new(3.0);
    /// client.send("name", message);
    /// ```
    pub fn send_packet(&mut self, packet_name: &str, packet: Packet) {
        self.client_packets.packets.insert(packet_name.to_owned(), packet);

        self.socket.try_send_to(&self.client_packets.serialize(), self.server_addr).ok();
    }
}