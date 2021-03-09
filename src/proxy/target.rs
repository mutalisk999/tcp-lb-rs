use md5;

use crate::proxy::connection::{get_target_conn_count_by_target_id};
use crate::proxy::proxy::{ProxyServer};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TargetDumpOrder {
    NoOrder,
    AscOrder,
    DescOrder,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub target_endpoint: String,
    pub target_active: bool,
    pub target_status: bool,
    pub target_max_conn: u32,
    pub target_timeout: u32,
}

impl Target {
    pub fn new(target_endpoint: String,
               target_max_conn: u32,
               target_timeout: u32,
               target_active: bool,
               target_status: bool) -> Target {
        Target {
            target_endpoint,
            target_active,
            target_status,
            target_max_conn,
            target_timeout,
        }
    }
}

pub fn calc_target_id_by_endpoint(endpoint: String) -> String {
    let digest = md5::compute(endpoint.as_str());
    format!("{:x}",digest).to_string()
}

pub async fn init_targets_from_config(proxy_server: &ProxyServer) {
    for target_config in proxy_server.server_config.lb_targets.iter() {
        let target= Target::new(
            target_config.target_endpoint.clone(),
            target_config.target_max_conn,
            target_config.target_timeout,
            target_config.target_active, true);

        proxy_server.targets_info.lock().await.insert(calc_target_id_by_endpoint(target.clone().target_endpoint),target);
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

pub async fn dump_targets(proxy_server: &ProxyServer, order: TargetDumpOrder) -> Vec<TargetDump> {
    let mut target_dump_vec = Vec::<TargetDump>::new();
    for (_, v) in proxy_server.targets_info.lock().await.iter(){
        let target_conn_count = get_target_conn_count_by_target_id(v.target_endpoint.clone(), proxy_server).await;
        let target_dump = TargetDump::new(v.target_endpoint.clone(), v.target_max_conn,  target_conn_count,
                                           v.target_timeout, v.target_active, v.target_status);
        target_dump_vec.push(target_dump);
    }
    match order {
        TargetDumpOrder::AscOrder => target_dump_vec.sort_by(|l, r| l.target_conn_count.cmp(&r.target_conn_count)),
        TargetDumpOrder::DescOrder => target_dump_vec.sort_by(|l, r| r.target_conn_count.cmp(&l.target_conn_count)),
        TargetDumpOrder::NoOrder => (),
    };
    target_dump_vec
}

#[test]
fn test_calc_target_id() {
    let target_id = calc_target_id_by_endpoint("127.0.0.1:1080".to_string());
    println!("target_id: {:?}", target_id);
}