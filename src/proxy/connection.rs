use std::collections::HashMap;
use tokio;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use uuid::Uuid;
use std::error::Error;

use crate::proxy::proxy::{ProxyServer};


#[derive(Debug, Clone)]
pub struct Connection {
    pub connect_id: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub start_time_1m: i64,
    pub start_time_5m: i64,
    pub start_time_30m: i64,
    pub read_bytes_1m: u64,
    pub read_bytes_5m: u64,
    pub read_bytes_30m: u64,
    pub write_bytes_1m: u64,
    pub write_bytes_5m: u64,
    pub write_bytes_30m: u64,
}

impl Connection {
    pub fn new(local_endpoint: String, remote_endpoint: String) -> Connection {
        Connection {
            connect_id: new_connection_id(),
            local_endpoint,
            remote_endpoint,
            start_time_1m: 0,
            start_time_5m: 0,
            start_time_30m: 0,
            read_bytes_1m: 0,
            read_bytes_5m: 0,
            read_bytes_30m: 0,
            write_bytes_1m: 0,
            write_bytes_5m: 0,
            write_bytes_30m: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub connection: Connection,
}

impl NodeConnection {
    pub fn new(local_endpoint: String, remote_endpoint: String) -> NodeConnection {
        NodeConnection {
            connection: Connection::new(local_endpoint, remote_endpoint),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TargetConnection {
    pub connection: Connection,
    pub target_id: String,
}

impl TargetConnection {
    pub fn new(local_endpoint: String, remote_endpoint: String, target_id: String) -> TargetConnection {
        TargetConnection {
            connection: Connection::new(local_endpoint, remote_endpoint), target_id,
        }
    }
}

pub async fn get_target_conn_count_by_target_id(target_id: String, proxy_server: &ProxyServer) -> u32 {
    let mut target_conn: u32 = 0;
    for (_, v) in proxy_server.connections_info.lock().await.iter() {
        if v.1.target_id == target_id {
            target_conn += 1;
        }
    }
    target_conn
}

pub fn new_connection_id() -> String {
    let connection_id = Uuid::new_v4();
    format!("{:x}",connection_id).to_string()
}

#[test]
fn test_new_connection_id() {
    for _ in (0..10) {
        let conn_id = new_connection_id();
        println!("connection_id: {:?}", conn_id);
    }
}