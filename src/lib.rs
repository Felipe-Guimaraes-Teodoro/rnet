use std::{net::SocketAddr, sync::Arc};

pub use prelude::*;
pub use tokio::*;

pub mod prelude;
pub mod udp_client;
pub mod udp_server;
pub mod packet;
pub mod tcp_server;
pub mod tcp_client;

use tokio::sync::Mutex;

pub const NET_BUFFER_SIZE: usize = 1024;

pub async fn test_server_udp() {
    println!("Initiated server!");

    let server = Arc::new(Mutex::new(Server::new("127.0.0.1:8080").await));
    let info = ServerUpdateInfo {
        tick_time_ms: 1,
        ..Default::default()
    };

    let server_clone = server.clone();
    task::spawn(async move {
        loop {
            let mut server_lock = server_clone.lock().await;
            server_lock.update(&info);
        }
    });
    

    let mut val = 0;
    loop {
        if let Ok(mut server) = server.try_lock() {
            server.send_packets_to_all_connected("y", Packet::new(val));

            val+=1;
        }
    }
}

pub async fn test_client_udp() {
    println!("Initiated client!");

    let mut client = UdpClient::new("0.0.0.0:0000", "127.0.0.1:8080").await;
    client.send_packet("greet", Packet::new("hi"));
    
    let mut i = 0;
    loop {
        
        if client.server_packets.packets.len() == 0 {
            if i % 500 == 0 {
                client.send_packet("greet", Packet::new("hi"));
            }
        }

        println!("{:?}", client.server_packets.get::<i32>("y").unwrap_or_default());

        client.update();
        i+=1;
    }
}


pub async fn test_server_tcp() {
    let mut server = TcpServer::new().await;
    server.server_loop().await;
}

pub async fn test_client_tcp() {
    let server_addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();

    let mut client = TcpClient::new(0, server_addr).await;
    
    loop {
        client.send_packet("HI!!!!!!").await;
        client.update().await;
    }

    // client.disconnect().await;
}