use std::collections::HashMap;
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::proxy::proxy::{ProxyServer};


#[derive(Debug)]
pub struct Connection {
    pub tcp_stream_read: Arc<Mutex<OwnedReadHalf>>,
    pub tcp_stream_write: Arc<Mutex<OwnedWriteHalf>>,
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
    pub fn new(tcp_stream_read: Arc<Mutex<OwnedReadHalf>>, tcp_stream_write: Arc<Mutex<OwnedWriteHalf>>) -> Connection {
        Connection {
            tcp_stream_read,
            tcp_stream_write,
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

#[derive(Debug)]
pub struct NodeConnection {
    pub connection: Connection,
}

impl NodeConnection {
    pub fn new(tcp_stream_read: Arc<Mutex<OwnedReadHalf>>, tcp_stream_write: Arc<Mutex<OwnedWriteHalf>>) -> NodeConnection {
        NodeConnection {
            connection: Connection::new(tcp_stream_read, tcp_stream_write),
        }
    }
}

#[derive(Debug)]
pub struct TargetConnection {
    pub connection: Connection,
    pub target_id: String,
}

impl TargetConnection {
    pub fn new(tcp_stream_read: Arc<Mutex<OwnedReadHalf>>, tcp_stream_write: Arc<Mutex<OwnedWriteHalf>>, target_id: String) -> TargetConnection {
        TargetConnection {
            connection: Connection::new(tcp_stream_read, tcp_stream_write),
            target_id,
        }
    }
}

pub async fn get_target_conn_count_by_target_id(target_id: String, proxy_server: &ProxyServer) -> u32 {
    let mut target_conn: u32 = 0;
    for (_, v) in proxy_server.connections_info.lock().await.iter() {
        if v.target_id == target_id {
            target_conn += 1;
        }
    }
    target_conn
}