use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use threadpool::ThreadPool;
use tokio::{net::UdpSocket, sync::Mutex};

use crate::{packet::{Packet, Packets}, NET_BUFFER_SIZE};

#[derive(Default)]
struct ClientState {
}

#[derive(Default)]
pub struct ServerUpdateInfo {
    pub tick_time_ms: u64,
    pub enable_auth: bool,
}

pub struct Server {
    socket: Arc<UdpSocket>,
    pub addr: SocketAddr,
    buf: [u8; NET_BUFFER_SIZE],
    clients: HashMap<SocketAddr, ClientState>,
    packets: Arc<Mutex<HashMap<SocketAddr, Packets>>>, 
    /*  
    packets recv from clients organized by connection; each packet is organized by name
    
    user1: [packet1, packet2, packet3]
    user2: [packet1, ]
    user3: []
     */
    pool: ThreadPool,

    count: u32,
}

impl Server {
    pub async fn new(server_addr: &str) -> Self {
        let addr = server_addr.parse::<SocketAddr>().unwrap();
        let socket: Arc<UdpSocket> = UdpSocket::bind(addr).await
            .unwrap()
            .into();
        
        Self { 
            pool: ThreadPool::new(4),
            socket,
            addr,
            buf: [0; NET_BUFFER_SIZE],
            clients: HashMap::new(),
            packets: Arc::new(Mutex::new(HashMap::new())),
            count: 0,
         }
    }

    pub fn update(&mut self, info: &ServerUpdateInfo) {
        let now = std::time::Instant::now();
        
        if let Ok((_, addr)) = self.socket.try_recv_from(&mut self.buf) {
            if self.clients.contains_key(&addr) {
                // let sock = self.socket.clone();
                let packets = self.packets.clone();
                let buf = self.buf;
                self.pool.execute(move || {
                    Self::handle_client(addr, packets, &buf);
                });
            } else {
                self.handle_connection(addr);
            }
        }

        std::thread::sleep(
            Duration::from_millis(info.tick_time_ms)
                .saturating_sub(now.elapsed())
        );
    }

    pub fn handle_client(addr: SocketAddr, packets: Arc<Mutex<HashMap<SocketAddr, Packets>>>, buf: &[u8; 1024]) {
        // store packages sent from client to server's packet storage
        if let Ok(mut packets) = packets.try_lock() {
            packets.insert(addr, Packet::deserialize(buf));
        }
    }

    pub fn send_packet(&mut self, addr: SocketAddr, packet_name: &str, packet: Packet) {
        if let Ok(mut packets) = self.packets.try_lock() {
            if let Some(ref mut client_packets) = packets.get_mut(&addr) {
                client_packets.packets.insert(packet_name.to_owned(), packet);

                self.socket.try_send_to(&client_packets.serialize(), addr).ok();
            }
        }
    }

    pub fn send_packets_to_all_connected(&mut self, packet_name: &str, packet: Packet) {
        if let Ok(mut packets) = self.packets.try_lock() {
            for (addr, client_packets) in packets.iter_mut() {
                client_packets.packets.insert(packet_name.to_owned(), packet.clone());

                self.socket.try_send_to(&client_packets.serialize(), *addr).ok();
            }
        }
    }

    pub fn handle_connection(&mut self, addr: SocketAddr) {
        self.clients.insert(addr, ClientState::default());
        dbg!("A client connected!;");
        if let Ok(mut packets) = self.packets.try_lock() {
            packets.insert(addr, Packets {
                packets: HashMap::new(),
            });
        }
        self.count+=1;
    }
}