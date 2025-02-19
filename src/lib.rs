use std::{net::SocketAddr, sync::Arc, time::Duration};

pub use prelude::*;
pub use tokio::*;

pub mod prelude;
pub mod udp_client;
pub mod udp_server;
pub mod packet;
pub mod tcp_server;
pub mod tcp_client;
pub mod gui;

use tokio::sync::{Mutex, RwLock};

pub const NET_BUFFER_SIZE: usize = 1024;

pub async fn test_server_udp() {
    println!("Initiated server!");

    let server = Arc::new(RwLock::new(Server::new("127.0.0.1:8080").await));

    let info = ServerUpdateInfo {
        tick_time_ms: 0,
        queue_task_amm: 4,
        concurrent_capacity: 4,
        pool_thd_count: 4,
        max_queue_len: 4,
        ..Default::default()
    };

    let server_clone = server.clone();
    task::spawn(async move {
        loop {
            if let Ok(mut server) = server_clone.try_write() {
                server.send_packets_to_all_connected("y", Packet::new(13));
                server.update(&info);
            }

            tokio::time::sleep(Duration::from_millis(16))
                .await;
        }
    });

    // gui feature
    let server_clone = server.clone();
    gui::server_window(server_clone).unwrap();
}

pub async fn test_client_udp() {
    println!("Initiated client!");

    let mut client = UdpClient::new("0.0.0.0:0000", "127.0.0.1:8080").await;
    client.send_packet("greet", Packet::new("hi"));
    
    let mut i = 0;
    loop {
        println!("{:?}", client.get::<i32>("y").unwrap_or_default());
        client.send_packet("greet", Packet::new("hi"));

        client.update();
        i+=1;

        tokio::time::sleep(Duration::from_millis(16))
            .await;
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