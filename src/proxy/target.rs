use crate::proxy::config::Config;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::error::Error;
use crate::proxy::connection::{get_target_conn_count_by_target_id, TargetConnection, NodeConnection};

#[derive(Debug, Clone)]
pub struct Target {
    pub target_endpoint: String,
    pub target_active: bool,
    pub target_status: bool,
    pub target_max_conn: u32,
    pub target_timeout: u32,
}

impl Target {
    pub fn new(target_endpoint: String, target_max_conn: u32, target_timeout: u32, target_active: bool, target_status: bool) -> Target {
        Target {
            target_endpoint,
            target_active,
            target_status,
            target_max_conn,
            target_timeout,
        }
    }
}

pub async fn init_targets_from_config(config: &Config, targets: Arc<Mutex<HashMap<String, Target>>>) {
    for target_config in config.lb_targets.iter() {
        let target= Target::new(target_config.target_endpoint.clone(),
                                target_config.target_max_conn, target_config.target_timeout,
                                target_config.target_active, true);
        targets.lock().await.insert(target.target_endpoint.clone(),target);
    }
}

#[derive(Debug, Clone)]
pub struct TargetDump {
    pub target: Target,
    pub target_conn_count: u32,
}

impl TargetDump {
    pub fn new(target_endpoint: String, target_max_conn: u32, target_conn_count: u32, target_timeout: u32, target_active: bool, target_status: bool) -> TargetDump {
        TargetDump {
            target: Target::new(target_endpoint, target_max_conn, target_timeout, target_active, target_status),
            target_conn_count,
        }
    }
}

pub async fn dump_targets(targets: Arc<Mutex<HashMap<String, Target>>>,
                          conn_pair_t2n: Arc<Mutex<HashMap<Arc<Mutex<TargetConnection>>, Arc<Mutex<NodeConnection>>>>>) -> Vec<TargetDump> {
    let mut target_dump_vec = Vec::<TargetDump>::new();
    for (_, v) in targets.lock().await.iter(){
        let target_conn_count = get_target_conn_count_by_target_id(v.target_endpoint.clone(), conn_pair_t2n.clone()).await;
        let target_dump = TargetDump::new(v.target_endpoint.clone(), v.target_max_conn,  target_conn_count,
                                           v.target_timeout, v.target_active, v.target_status);
        target_dump_vec.push(target_dump);
    }
    target_dump_vec.sort_by(|l,r| l.target_conn_count.cmp(&r.target_conn_count));
    target_dump_vec
}
