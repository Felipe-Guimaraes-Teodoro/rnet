use rnet::{test_client_tcp, test_client_udp, test_many_client_udp, test_server_tcp, test_server_udp};

#[tokio::main]
async fn main() {
    let env: Vec<String> = std::env::args().collect();

    if env.contains(&"udp".to_owned()) {
        if env.contains(&"server".to_owned()) {
            test_server_udp().await;
        } else {
            if env.contains(&"many".to_owned()) { 
                test_many_client_udp().await;
            } else {
                test_client_udp().await;
            }
        }
    }
    if env.contains(&"tcp".to_owned()) {
        if env.contains(&"server".to_owned()) {
            test_server_tcp().await;
        } else {
            test_client_tcp().await;
        }
    }

    loop {}
}