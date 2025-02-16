use std::{collections::HashMap, net::{Ipv4Addr, SocketAddr}, sync::Arc};
use tokio::net::UdpSocket;

use crate::packet::{Packet, Packets};
pub const NET_BUFFER_SIZE: usize = 1024;

// I'd tell you a UDP joke... but you might not get it

pub struct UdpClient {
    id: u32,
    socket: UdpSocket,
    our_addr: SocketAddr,
    server_addr: SocketAddr,
    buf: [u8; NET_BUFFER_SIZE],

    pub client_packets: Packets, // client's packets
    pub server_packets: Packets, // packets received from srever
}

impl UdpClient {
    pub async fn new(id: u32, server_addr: SocketAddr) -> Self {
        let our_address = "0.0.0.0:0000".parse::<SocketAddr>().unwrap();
        let socket: UdpSocket = UdpSocket::bind(our_address).await.unwrap();

        let buf = [0 as u8; NET_BUFFER_SIZE];
        
        Self {
            id, 
            socket, our_addr: our_address, 
            server_addr, buf, 
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