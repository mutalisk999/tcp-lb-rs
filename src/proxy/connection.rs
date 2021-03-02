use std::collections::HashMap;

use tokio::net::TcpStream;


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