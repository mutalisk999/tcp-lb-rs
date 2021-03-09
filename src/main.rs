use tokio;

mod proxy;
use proxy::target::{init_targets_from_config};
use proxy::proxy::{ProxyServer, start_tcp_proxy_server};
use proxy::api::{start_api_server};


async fn run(proxy_server: &ProxyServer) {
    // make sure config parameters valid
    let _ = proxy_server.server_config.check();

    // init targets
    init_targets_from_config(proxy_server).await;
    println!("targets init, count: {}", proxy_server.targets_info.lock().await.len());

    let fut_tcp_proxy_server = start_tcp_proxy_server(proxy_server);
    println!("starting tcp proxy server...");

    let fut_api_server = start_api_server(proxy_server);
    println!("starting api server...");

    let (_, _) = tokio::join!(fut_tcp_proxy_server, fut_api_server);
}

#[tokio::main]
async fn main() {
    let mut server_info = ProxyServer::new();
    run(&mut server_info).await;
}