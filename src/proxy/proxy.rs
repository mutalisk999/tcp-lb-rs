use tokio;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::error::Error;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use std::borrow::{BorrowMut, Borrow};
use tokio::net::tcp::{ReadHalf, WriteHalf};

use crate::proxy::config::Config;
use crate::proxy::connection::{NodeConnection, TargetConnection, new_connection_id};
use crate::proxy::target::{Target, TargetDumpOrder, calc_target_id_by_endpoint, dump_targets};
use crate::proxy::config::{read_config};
use std::ops::Deref;


#[derive(Debug)]
pub struct ProxyServer {
    pub server_config: Config,
    pub targets_info: Arc<tokio::sync::Mutex<HashMap<String, Target>>>,
    pub connections_info: Arc<tokio::sync::Mutex<HashMap<String, (NodeConnection, TargetConnection)>>>,
}

impl ProxyServer {
    pub fn new() -> ProxyServer{
        ProxyServer {
            server_config: read_config(),
            targets_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            connections_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

pub async fn start_tcp_proxy(proxy_server: &mut ProxyServer
    ) -> Result<(), Box<dyn Error>>{

    let node_listener = tokio::net::TcpListener::bind(proxy_server.server_config.lb_node.listen.as_str())
        .await.expect(format!("Failure binding node listen endpoint [{}]", proxy_server.server_config.lb_node.listen).as_str());

    loop {
        let (mut tcp_stream_node, node_remote_addr) = node_listener.accept().await?;
        println!("remote connection from {}", node_remote_addr);

        let targets_dump = dump_targets(proxy_server, TargetDumpOrder::AscOrder).await;

        let mut socket_conn: Option<tokio::net::TcpSocket> = None;
        let mut tcp_stream_target: Option<tokio::net::TcpStream> = None;
        let mut conn_target_info: Option<Target> = None;

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
            tcp_stream_target = match r {
                Ok(c) => Some(c),
                Err(_) => continue
            };

            conn_target_info = Some(t.target.clone());
            break;
        }

        match tcp_stream_target {
            Some(_) => (),
            None => {
                let _ = tcp_stream_node.shutdown().await;
                continue;
            }
        }
        let conn_target_id = calc_target_id_by_endpoint(conn_target_info.clone().unwrap().target_endpoint);
        let mut tcp_stream_target = tcp_stream_target.unwrap();
        let target_local_addr = tcp_stream_target.local_addr().unwrap().to_string();

        let (mut tcp_stream_node_read, mut tcp_stream_node_write) = tcp_stream_node.into_split();
        let (mut tcp_stream_target_read, mut tcp_stream_target_write) = tcp_stream_target.into_split();

        let node_connection_info = NodeConnection::new(
            proxy_server.server_config.lb_node.listen.clone(),
            node_remote_addr.to_string());

        let target_connection_info = TargetConnection::new(
            target_local_addr,
            conn_target_info.clone().unwrap().target_endpoint,
            conn_target_id
        );

        let proxy_connection_id = new_connection_id();
        let mut connection_info_arc = proxy_server.connections_info.clone();
        let proxy_connection_id_dump = proxy_connection_id.clone();
        let mut connection_info_arc_dump = connection_info_arc.clone();

        let mut connection_info = proxy_server.connections_info.lock().await.clone();
        connection_info.insert(proxy_connection_id.clone(), (node_connection_info, target_connection_info));

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match tcp_stream_node_read.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        connection_info_arc.lock().await.remove(&proxy_connection_id);
                        eprintln!("|{}| tcp_stream_node_read closed by remote", proxy_connection_id);
                        return;
                    },
                    Ok(n) => {
                        {
                            let node_info = connection_info_arc.lock().await;
                            let v = node_info.get(&proxy_connection_id);
                            match v {
                                Some((node_info, target_info)) => {
                                    let mut node_info_dump = node_info.clone();
                                    let mut target_info_dump = target_info.clone();
                                    node_info_dump.connection.read_bytes_1m += n as u64;
                                    node_info_dump.connection.read_bytes_5m += n as u64;
                                    node_info_dump.connection.read_bytes_30m += n as u64;
                                    connection_info_arc.lock().await.insert(proxy_connection_id.clone(), (node_info_dump, target_info_dump));
                                },
                                None => {}
                            }
                        }
                        n
                    },
                    Err(e) => {
                        connection_info_arc.lock().await.remove(&proxy_connection_id.clone());
                        eprintln!("|{}| failed to read from socket; err = {:?}", proxy_connection_id, e);
                        return;
                    }
                };

                if let Err(e) = tcp_stream_target_write.write_all(&buf[0..n]).await {
                    connection_info_arc.lock().await.remove(&proxy_connection_id);
                    eprintln!("|{}| failed to write to socket; err = {:?}", proxy_connection_id, e);
                    return;
                } else {
                    {
                        let node_info = connection_info_arc.lock().await;
                        let v = node_info.get(&proxy_connection_id);
                        match v {
                            Some((node_info, target_info)) => {
                                let mut node_info_dump = node_info.clone();
                                let mut target_info_dump = target_info.clone();
                                target_info_dump.connection.write_bytes_1m += n as u64;
                                target_info_dump.connection.write_bytes_5m += n as u64;
                                target_info_dump.connection.write_bytes_30m += n as u64;
                                connection_info_arc.lock().await.insert(proxy_connection_id.clone(), (node_info_dump, target_info_dump));
                            },
                            None => {}
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match tcp_stream_target_read.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        connection_info_arc_dump.lock().await.remove(&proxy_connection_id_dump);
                        eprintln!("|{}| tcp_stream_target_read closed by remote", proxy_connection_id_dump);
                        return
                    },
                    Ok(n) => {
                        {
                            let node_info = connection_info_arc_dump.lock().await;
                            let v = node_info.get(&proxy_connection_id_dump);
                            match v {
                                Some((node_info, target_info)) => {
                                    let mut node_info_dump = node_info.clone();
                                    let mut target_info_dump = target_info.clone();
                                    target_info_dump.connection.read_bytes_1m += n as u64;
                                    target_info_dump.connection.read_bytes_5m += n as u64;
                                    target_info_dump.connection.read_bytes_30m += n as u64;
                                    connection_info_arc_dump.lock().await.insert(proxy_connection_id_dump.clone(), (node_info_dump, target_info_dump));
                                },
                                None => {}
                            }
                        }
                        n
                    },
                    Err(e) => {
                        connection_info_arc_dump.lock().await.remove(&proxy_connection_id_dump);
                        eprintln!("|{}| failed to read from socket; err = {:?}", proxy_connection_id_dump, e);
                        return;
                    }
                };

                if let Err(e) = tcp_stream_node_write.write_all(&buf[0..n]).await {
                    connection_info_arc_dump.lock().await.remove(&proxy_connection_id_dump);
                    eprintln!("|{}| failed to write to socket; err = {:?}", proxy_connection_id_dump, e);
                    return;
                } else {
                    {
                        let node_info = connection_info_arc_dump.lock().await;
                        let v = node_info.get(&proxy_connection_id_dump);
                        match v {
                            Some((node_info, target_info)) => {
                                let mut node_info_dump = node_info.clone();
                                let mut target_info_dump = target_info.clone();
                                node_info_dump.connection.write_bytes_1m += n as u64;
                                node_info_dump.connection.write_bytes_5m += n as u64;
                                node_info_dump.connection.write_bytes_30m += n as u64;
                                connection_info_arc_dump.lock().await.insert(proxy_connection_id_dump.clone(), (node_info_dump, target_info_dump));
                            },
                            None => {}
                        }
                    }
                }
            }
        });
    }
}
