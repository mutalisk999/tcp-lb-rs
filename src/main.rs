#[macro_use]
extern crate lazy_static;
extern crate tokio;

mod proxy;
use proxy::target::{init_targets_from_config};
use proxy::proxy::{start_tcp_proxy_server};
use proxy::connection::{start_maintain_loop};
use proxy::api::{start_api_server};
use std::ops::Deref;
use log::{info};

use proxy::g::SERVER_INFO;
use flexi_logger::{Duplicate, detailed_format};

fn init_log() {
    flexi_logger::Logger::with_str(SERVER_INFO.deref().server_config.lb_log.log_set_level.clone())
        .log_to_file()
        .directory("log")
        .basename("tcp_lb_rs.log")
        .duplicate_to_stdout(Duplicate::All)
        .format_for_files(detailed_format)
        .format_for_stdout(detailed_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
}

async fn run() {
    // make sure config parameters valid
    let _ = SERVER_INFO.deref().server_config.check();

    // init log
    init_log();

    // init targets
    init_targets_from_config().await;
    info!("targets init, count: {}", SERVER_INFO.deref().targets_info.lock().await.len());

    let fut_tcp_proxy_server = start_tcp_proxy_server();
    info!("starting tcp proxy server, listen on [{}]...", SERVER_INFO.deref().server_config.lb_node.listen.clone());

    let fut_api_server = start_api_server();
    info!("starting api server, listen on [{}]... ", SERVER_INFO.deref().server_config.lb_api.listen.clone());

    let fut_maintain_loop = start_maintain_loop();
    info!("starting maintain loop...");

    let (_, _, _) = tokio::join!(fut_tcp_proxy_server, fut_api_server, fut_maintain_loop);
}

#[tokio::main]
async fn main() {
    run().await;
}