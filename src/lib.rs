use std::{arch::x86_64::_rdrand16_step, collections::HashMap, net::{Ipv4Addr, SocketAddr, SocketAddrV4}};

use prelude::*;

pub mod prelude;
pub mod udp_client;
pub mod udp_server;
pub mod packet;
pub mod tcp_server;
pub mod tcp_client;

pub async fn test_server_udp() {
    println!("Initiated server!");

    let mut server = Server::new().await;

    let mut val = 0;
    loop {
        server.send_packets_to_all_connected("y", Packet::new(13));

        if unsafe { _rdrand16_step(&mut val) } > 0 {
            if val as f32 / u16::MAX as f32 > 0.99 {
                server.send_packets_to_all_connected("pos", Packet::new([0.0f32, 1.0, 0.0]));
            }
        }

        server.send_packets_to_all_connected("name", Packet::new("felipe".to_string()));

        server.update();
    }
}

pub async fn test_client_udp() {
    let server_addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();

    println!("Initiated client!");

    let mut client = UdpClient::new(0, server_addr).await;
    client.send_packet("greet", Packet::new("hi"));
    
    loop{
        let str = client.server_packets.get::<i32>("y");
        if let Some(str) = str {
            dbg!(str);
        }

        let str = client.server_packets.get::<[f32; 3]>("pos");
        if let Some(str) = str {
            dbg!(str);
        }

        let str = client.server_packets.get::<String>("name");
        if let Some(str) = str {
            dbg!(str);
        }
        client.update();
    }
}


pub async fn test_server_tcp() {
    let mut server = TcpServer::new().await;
    server.server_loop().await;
}

pub async fn test_client_tcp() {
    let server_addr = "127.0.0.2:8080".parse::<SocketAddr>().unwrap();

    let mut client = TcpClient::new(0, server_addr).await;
    
    loop {
        client.listen().await;
        client.send_msg("HI!!!!!!").await;
    }

    client.disconnect().await;
}
