use tokio;

mod proxy;
use proxy::proxy::{ProxyServer, start_tcp_proxy};
use proxy::target::{init_targets_from_config};

async fn run(proxy_server: &mut ProxyServer) {
    // make sure config parameters valid
    let _ = proxy_server.server_config.check();

    // init targets
    init_targets_from_config(proxy_server).await;
    println!("init targets count: {}", proxy_server.targets_info.lock().await.len());

    let fut_tcp_proxy = start_tcp_proxy(proxy_server);

    let _ = tokio::join!(fut_tcp_proxy,);
}

#[tokio::main]
async fn main() {
    let mut server_info = ProxyServer::new();
    run(&mut server_info).await;
}