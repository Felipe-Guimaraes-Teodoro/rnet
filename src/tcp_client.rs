use std::{collections::HashMap, net::SocketAddr};
use tokio::{io::AsyncWriteExt, net::{TcpSocket, TcpStream}};

use crate::{Packets, NET_BUFFER_SIZE};

pub struct TcpClient {
    id: u32,
    stream: TcpStream,
    server_addr: SocketAddr,
    buf: [u8; NET_BUFFER_SIZE],
    packets: HashMap<SocketAddr, Packets>,
}

impl TcpClient {
    pub async fn new(id: u32, server_addr: SocketAddr) -> Self {
        let socket: TcpSocket = TcpSocket::new_v4().unwrap();
        let stream = socket.connect(server_addr).await.unwrap();

        println!("CLIENT: Client connected to {}...", server_addr);

        Self {
            id,
            stream,
            server_addr,
            buf: [0; NET_BUFFER_SIZE],
            packets: HashMap::new(),
        }
    }

    pub async fn send_packet(&mut self, msg: &str){
        self.stream.writable().await.unwrap();

        match self.stream.try_write(msg.as_bytes()) {
            Ok(_) => {
                println!("CLIENT: Sent {} to {:?}", msg, self.server_addr);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                eprintln!("couldnt");
            }
            Err(e) => {
                println!("CLIENT: Failed to write to {:?}: {}", self.server_addr, e)
            }
        }
    }

    pub async fn update(&mut self){
        self.stream.readable().await.unwrap();

        match self.stream.try_read(&mut self.buf) {
            Ok(0) => return,
            Ok(n) => {
                if let Ok(msg) = std::str::from_utf8(&self.buf[0..n]){
                    println!("CLIENT: Read \"{}\"!", msg);
                }
                else{
                    eprint!("CLIENT: Failed to convert bytes to message: {}", n);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // noop
            }
            Err(e) => {
                println!("CLIENT: Failed to read from {:?}: {}", self.server_addr, e)
            }
        }
    }

    pub async fn disconnect(&mut self){
        self.send_packet("DISCONNECTING").await;

        println!("CLIENT: Disconnecting client {}", self.id);
        self.stream.shutdown().await.unwrap();
    }
}
