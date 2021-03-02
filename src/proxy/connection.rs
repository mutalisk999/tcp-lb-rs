use std::collections::HashMap;

use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;


#[derive(Debug)]
pub struct Connection {
    pub tcp_stream: TcpStream,
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
    pub fn new(tcp_stream: TcpStream) -> Connection {
        Connection {
            tcp_stream,
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
    pub fn new(tcp_stream: TcpStream) -> NodeConnection {
        NodeConnection {
            connection: Connection::new(tcp_stream),
        }
    }
}

#[derive(Debug)]
pub struct TargetConnection {
    pub connection: Connection,
    pub target_id: String,
}

impl TargetConnection {
    pub fn new(tcp_stream: TcpStream, target_id: String) -> TargetConnection {
        TargetConnection {
            connection: Connection::new(tcp_stream),
            target_id,
        }
    }
}

pub async fn get_target_conn_count_by_target_id(target_id: String,
                                                conn_pair_t2n: Arc<Mutex<HashMap<Arc<Mutex<TargetConnection>>, Arc<Mutex<NodeConnection>>>>>) -> u32 {
    let mut target_conn: u32 = 0;
    for (k, _) in conn_pair_t2n.lock().await.iter() {
        if k.lock().await.target_id == target_id {
            target_conn += 1;
        }
    }
    target_conn
}