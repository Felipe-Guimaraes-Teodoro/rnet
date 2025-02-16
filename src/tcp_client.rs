// I'd tell you a UDP joke, but you might not get it...
use std::{collections::HashMap, net::{Ipv4Addr, SocketAddr}, task};
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpSocket, TcpStream}};

use crate::{packet::DeserializedPackets};

pub const NET_BUFFER_SIZE: usize = 1024;



/*
TODO: 
    server.send_client(client, data)
    client.send_server(server, data)

    # maybe
    have everything on a table
    @server->clients[i]
    ["player position"] = vec3
    ["custom client packet"] = CustomPacket

    @client->server_data
    ["other_player_positions"] = vec![vec3]

*/
pub struct TcpClient {
    id: u32,
    stream: TcpStream,
    server_addr: SocketAddr,
    buf: [u8; NET_BUFFER_SIZE],
    packets: HashMap<SocketAddr, DeserializedPackets>, // packets["client1", {["positions"], ..}];
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

    pub async fn send_msg(&mut self, msg: &str){
        loop{
            self.stream.writable().await.unwrap();

            match self.stream.try_write(msg.as_bytes()) {
                Ok(_) => {
                    println!("CLIENT: Sent {} to {:?}", msg, self.server_addr);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("CLIENT: Failed to write to {:?}: {}", self.server_addr, e)
                }
            }
        }
    }

    pub async fn listen(&mut self){
        loop{
            self.stream.readable().await.unwrap();

            match self.stream.try_read(&mut self.buf) {
                Ok(0) => break,
                Ok(n) => {
                    println!("CLIENT: Read {} bytes!", n);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("CLIENT: Failed to read from {:?}: {}", self.server_addr, e)
                }
            }
        }
    }

    pub async fn disconnect(&mut self){
        println!("CLIENT: Disconnecting client {}", self.id);
        self.stream.shutdown().await.unwrap();
    }
}
