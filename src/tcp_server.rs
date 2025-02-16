// I'd tell you a UDP joke, but you might not get it...
use std::{collections::HashMap, net::{Ipv4Addr, SocketAddr}, sync::Arc, task};
use threadpool::ThreadPool;
use tokio::net::{TcpSocket, TcpListener, TcpStream};

use crate::{packet::DeserializedPackets, prelude::UdpClient};

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
pub struct TcpServer {
    socket: Arc<TcpSocket>,
    pub addr: SocketAddr,
    listener: TcpListener,
    buf: [u8; NET_BUFFER_SIZE],
    // if this cant be serializable, do vec, and make packet have a name (or type idk)
    clients: Vec<SocketAddr>,
    packets: HashMap<SocketAddr, DeserializedPackets>, // packets["client1", {["positions"], ..}];
    pool: ThreadPool,
}

impl TcpServer {
    pub async fn new() -> Self {
        let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let socket: Arc<TcpSocket> = TcpSocket::new_v4().unwrap().into();
        socket.bind(addr).unwrap();

        println!("SERVER: Server listening on {}...", addr);

        let listener = TcpListener::bind(addr).await.unwrap();
        
        Self { 
            pool: ThreadPool::new(4),
            socket,
            addr,
            listener,
            buf: [0; NET_BUFFER_SIZE],
            clients: Vec::new(),
            packets: HashMap::new(),
        }
    }

    pub async fn server_loop(&mut self) {
        loop {
            self.handle_connection();
        }
    }

    async fn handle_connection(&mut self){
        match self.listener.accept().await{
            Ok((socket, addr)) => {
                println!("SERVER: New client {} connected!", addr);
                self.clients.push(addr);
                tokio::spawn(handle_client(socket)).await;
            }
            Err(e) => {
                println!("SERVER: Failed to accept client connection: {}", e);
            }
        }
    }
}

async fn handle_client(socket: TcpStream) {
    let mut buf = [0; 1024];
    let addr = socket.local_addr().unwrap();

    // store packages sent from client to our packet storage
    loop{
        match socket.try_read(&mut buf) {
            Ok(0) => {
                println!("SERVER: Client {} disconnected!", addr);
            }
            Ok(n) => {
                println!("SERVER: Read {} bytes!", n);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("SERVER: Failed to read from {:?}: {}", addr, e)
            }
        }

        match socket.try_write(b"Hey, my little clients!") {
            Ok(n) => {
                println!("SERVER: Wrote {} to {}!", n, addr);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("SERVER: Failed to write to {:?}: {}", addr, e)
            }
        }
    }
}
