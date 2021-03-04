use tokio;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::error::Error;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use std::borrow::{BorrowMut, Borrow};
use tokio::net::tcp::{ReadHalf, WriteHalf};

use crate::proxy::config::Config;
use crate::proxy::connection::{NodeConnection, TargetConnection};
use crate::proxy::target::{Target, TargetDumpOrder, dump_targets};
use crate::proxy::config::{read_config};


#[derive(Debug)]
pub struct ProxyServer {
    pub server_config: Config,
    pub targets_info: Arc<tokio::sync::Mutex<HashMap<String, Target>>>,
    pub connections_info: Arc<tokio::sync::Mutex<Vec<(NodeConnection, TargetConnection)>>>,
}

impl ProxyServer {
    pub fn new() -> ProxyServer{
        ProxyServer {
            server_config: read_config(),
            targets_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            connections_info: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}


pub async fn start_tcp_proxy(proxy_server: &mut ProxyServer
    ) -> Result<(), Box<dyn Error>>{

    let node_listener = tokio::net::TcpListener::bind(proxy_server.server_config.lb_node.listen.as_str())
        .await.expect(format!("Failure binding node listen endpoint [{}]", proxy_server.server_config.lb_node.listen).as_str());

    loop {
        let (mut tcp_stream_accept, remote_addr) = node_listener.accept().await?;
        println!("remote connection from {}", remote_addr);

        let targets_dump = dump_targets(proxy_server, TargetDumpOrder::AscOrder).await;

        let mut socket_conn: Option<tokio::net::TcpSocket> = None;
        let mut tcp_stream_conn: Option<tokio::net::TcpStream> = None;
        let mut conn_target_id: Option<String> = None;
        let mut target_time_out: Option<u32> = None;

        for t in targets_dump.iter() {
            if !t.target.target_active {
                continue;
            }
            if t.target_conn_count > t.target.target_max_conn {
                continue;
            }

            let r = tokio::net::TcpSocket::new_v4();
            socket_conn = match r {
                Ok(s) => Some(s),
                Err(_) => continue
            };
            let r = socket_conn.unwrap().connect(t.target.target_endpoint.parse().unwrap()).await;
            tcp_stream_conn = match r {
                Ok(c) => Some(c),
                Err(_) => continue
            };

            conn_target_id = Some(t.target.target_endpoint.clone());
            target_time_out = Some(t.target.target_timeout);
            break;
        }

        match tcp_stream_conn {
            Some(_) => (),
            None => {
                let _ = tcp_stream_accept.shutdown().await;
                continue;
            }
        }
        let conn_target_id = conn_target_id.unwrap();
        let mut tcp_stream_conn = tcp_stream_conn.unwrap();

        let (mut tcp_stream_accept_read, mut tcp_stream_accept_write) = tcp_stream_accept.into_split();
        let (mut tcp_stream_conn_read, mut tcp_stream_conn_write) = tcp_stream_conn.into_split();

        let accept_connection_read = Arc::new(tokio::sync::Mutex::new(tcp_stream_accept_read));
        let accept_connection_write = Arc::new(tokio::sync::Mutex::new(tcp_stream_accept_write));
        let conn_connection_read  = Arc::new(tokio::sync::Mutex::new(tcp_stream_conn_read));
        let conn_connection_write  = Arc::new(tokio::sync::Mutex::new(tcp_stream_conn_write));

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match accept_connection_read.lock().await.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                if let Err(e) = conn_connection_write.lock().await.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match conn_connection_read.lock().await.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                if let Err(e) = accept_connection_write.lock().await.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
