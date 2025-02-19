use std::{collections::{HashMap, VecDeque}, net::SocketAddr, sync::Arc, time::Duration};
use threadpool::ThreadPool;
use tokio::{net::UdpSocket, sync::Mutex};

use crate::{gui, packet::{Packet, Packets}, NET_BUFFER_SIZE};

#[derive(Default)]
pub struct ClientState {
}

#[derive(Default)]
pub struct ServerUpdateInfo {
    pub tick_time_ms: u64,
    pub queue_task_amm: u64,
    pub concurrent_capacity: u64,
    pub enable_auth: bool,
    pub max_queue_len: usize,
    pub pool_thd_count: usize,
}

#[derive(Clone, Debug)]
pub enum ServerCommand {
    Send(SocketAddr, String, Packet),
    SendAll(String, Packet),
}

pub struct Server {
    pub socket: Arc<UdpSocket>,
    pub packets: Arc<Mutex<HashMap<SocketAddr, Packets>>>, 
    pub addr: SocketAddr,
    pub buf: [u8; NET_BUFFER_SIZE],
    pub clients: HashMap<SocketAddr, ClientState>,
    /*  
    packets recv from clients organized by connection; each packet is organized by name
    
    user1: [packet1, packet2, packet3]
    user2: [packet1, ]
    user3: []
     */
    pub pool: ThreadPool,
    pub queue: VecDeque<ServerCommand>,

    count: u32,
}

impl Server {
    pub async fn new(server_addr: &str) -> Self {
        let addr = server_addr.parse::<SocketAddr>().unwrap();
        let socket: Arc<UdpSocket> = UdpSocket::bind(addr).await
            .unwrap()
            .into();
        
        let server = Self { 
            pool: ThreadPool::new(4),
            socket,
            addr,
            buf: [0; NET_BUFFER_SIZE],
            clients: HashMap::new(),
            packets: Arc::new(Mutex::new(HashMap::new())),
            count: 0,
            queue: VecDeque::new(),
         };

         server
    }

    pub fn update(&mut self, info: &ServerUpdateInfo) {
        let now = std::time::Instant::now();

        self.pool.set_num_threads(info.pool_thd_count);
        
        for _ in 0..info.concurrent_capacity {
            if let Ok((_, addr)) = self.socket.try_recv_from(&mut self.buf) {
                if self.clients.contains_key(&addr) {
                    let packets = self.packets.clone();
                    let buf = self.buf;
    
                    self.pool.execute(move || {
                        Self::handle_client(addr, packets, &buf);
                    });
                } else {
                    self.handle_connection(addr);
                }
    
                self.handle_queue(info);
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
        self.queue.push_back(ServerCommand::Send(addr, packet_name.to_owned(), packet));
    }

    pub fn send_packets_to_all_connected(&mut self, packet_name: &str, packet: Packet) {
        self.queue.push_back(ServerCommand::SendAll(packet_name.to_owned(), packet));
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

    pub fn handle_queue(&mut self, info: &ServerUpdateInfo) {
        for _ in 0..info.queue_task_amm {
            if let Some(command) = self.queue.pop_front() {
                match command {
                    ServerCommand::Send(addr, name, packet) => {
                        let socket = self.socket.clone();
                        let packets = self.packets.clone();

                        self.pool.execute(move || {
                            ServerCommand::send(socket, packets, addr, name, packet);
                        });
                    }
                    ServerCommand::SendAll(name, packet) => {
                        let socket = self.socket.clone();
                        let packets = self.packets.clone();

                        self.pool.execute(move || {
                            ServerCommand::send_all(socket, packets, name, packet);
                        });
                    }
                }
            }

            if self.queue.len() > info.max_queue_len {
                self.queue.pop_front();
            }
        }
    }
}

impl ServerCommand {
    fn send(
        socket: Arc<UdpSocket>,
        packets: Arc<Mutex<HashMap<SocketAddr, Packets>>>,
        addr: SocketAddr, 
        name: String, 
        packet: Packet
    ) {
        if let Ok(mut packets) = packets.try_lock() {
            if let Some(ref mut client_packets) = packets.get_mut(&addr) {
                client_packets.packets.insert(name, packet);
                socket.try_send_to(&client_packets.serialize(), addr).ok();
            }
        }
    }

    fn send_all(
        socket: Arc<UdpSocket>,
        packets: Arc<Mutex<HashMap<SocketAddr, Packets>>>,
        name: String, 
        packet: Packet
    ) {
        if let Ok(mut packets) = packets.try_lock() {
            for (addr, client_packets) in packets.iter_mut() {
                client_packets.packets.insert(name.to_owned(), packet.clone());

                socket.try_send_to(&client_packets.serialize(), *addr).ok();
            }
        }
    }
}