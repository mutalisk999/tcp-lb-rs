extern crate tokio;

use tokio::sync::Mutex;
use std::collections::HashMap;

mod proxy;
use proxy::config::{Config, read_config};
use proxy::proxy::{start_tcp_proxy};
use proxy::target::{Target, init_targets_from_config};
use proxy::connection::{NodeConnection, TargetConnection};
use std::sync::Arc;

async fn run() {
    let config: Config = read_config();
    let targets: Arc<Mutex<HashMap<String, Target>>> = Arc::new(Mutex::new(HashMap::new()));
    let conn_pair_n2t: Arc<Mutex<HashMap<Arc<Mutex<NodeConnection>>, Arc<Mutex<TargetConnection>>>>>
        = Arc::new(Mutex::new(HashMap::new()));
    let conn_pair_t2n: Arc<Mutex<HashMap<Arc<Mutex<TargetConnection>>, Arc<Mutex<NodeConnection>>>>>
        = Arc::new(Mutex::new(HashMap::new()));

    // make sure config parameters valid
    let _ = config.check();

    // init targets
    init_targets_from_config(&config, targets.clone()).await;
    println!("init targets count: {}", targets.lock().await.len());

    let fut_tcp_proxy = start_tcp_proxy(&config,
                                        targets.clone(),
                                        conn_pair_n2t.clone(),
                                        conn_pair_t2n.clone());

    let (_) = tokio::join!(fut_tcp_proxy,);
}

#[tokio::main]
async fn main() {
    run().await;
}