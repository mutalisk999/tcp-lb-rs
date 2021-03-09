use uuid::Uuid;

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

    pub fn add_read_n(&mut self, read_n: u64) {
        self.read_bytes_1m += read_n.clone();
        self.read_bytes_5m += read_n.clone();
        self.read_bytes_30m += read_n.clone();
    }

    pub fn add_write_n(&mut self, write_n: u64) {
        self.write_bytes_1m += write_n.clone();
        self.write_bytes_5m += write_n.clone();
        self.write_bytes_30m += write_n.clone();
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

    pub fn add_read_n(&mut self, read_n: u64) {
        self.connection.add_read_n(read_n);
    }

    pub fn add_write_n(&mut self, write_n: u64) {
        self.connection.add_write_n(write_n);
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

    pub fn add_read_n(&mut self, read_n: u64) {
        self.connection.add_read_n(read_n);
    }

    pub fn add_write_n(&mut self, write_n: u64) {
        self.connection.add_write_n(write_n);
    }
}

pub async fn get_target_conn_count_by_target_id(target_id: String, proxy_server: &ProxyServer) -> u32 {
    let mut target_conn: u32 = 0;
    for (_, v) in proxy_server.tunnel_info.lock().await.iter() {
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

pub fn new_tunnel_id() -> String {
    let tunnel_id = Uuid::new_v4();
    format!("{:x}",tunnel_id).to_string()
}

#[test]
fn test_new_connection_id() {
    for _ in 0..10 {
        let conn_id = new_connection_id();
        println!("connection_id: {:?}", conn_id);
    }
}

#[test]
fn test_new_tunnel_id() {
    for _ in 0..10 {
        let tunnel_id = new_tunnel_id();
        println!("tunnel_id: {:?}", tunnel_id);
    }
}