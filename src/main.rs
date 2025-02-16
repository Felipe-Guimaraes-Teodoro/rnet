use rnet::{test_client_tcp, test_client_udp, test_server_tcp, test_server_udp};

#[tokio::main]
async fn main() {
    let env: Vec<String> = std::env::args().collect();

    if env.contains(&"server_udp".to_owned()) {
        test_server_udp().await;
    }  
    if env.contains(&"client_udp".to_owned()){
        test_client_udp().await;
    }
    if env.contains(&"server_tcp".to_owned()){
        test_server_tcp().await;
    }
    if env.contains(&"client_tcp".to_owned()){
        test_client_tcp().await;
    }
}