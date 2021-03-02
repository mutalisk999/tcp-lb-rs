use crate::proxy::config::Config;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Target {
    pub target_endpoint: String,
    pub target_active: bool,
    pub target_status: bool,
    pub target_max_conn: u32,
    pub target_timeout: u32,
}

impl Target {
    pub fn new(target_endpoint: String, target_max_conn: u32, target_timeout: u32) -> Target {
        Target {
            target_endpoint,
            target_active: true,
            target_status: true,
            target_max_conn,
            target_timeout,
        }
    }
}

pub async fn init_targets_from_config(config: &Config, targets: Arc<Mutex<HashMap<String, Target>>>) {
    for target_config in config.lb_targets.iter() {
        let target= Target::new(target_config.target_endpoint.clone(), target_config.target_max_conn, target_config.target_timeout);
        targets.lock().await.insert(target.target_endpoint.clone(),target);
    }
}

#[derive(Debug, Clone)]
pub struct TargetDump {
    pub target: Target,
    pub target_conn_count: u32,
}

impl TargetDump {
    pub fn new(target_endpoint: String, target_max_conn: u32, target_conn_count: u32, target_timeout: u32) -> TargetDump {
        TargetDump {
            target: Target::new(target_endpoint, target_max_conn, target_timeout),
            target_conn_count,
        }
    }
}
