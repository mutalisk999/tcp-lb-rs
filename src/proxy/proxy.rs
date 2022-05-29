use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{Ordering};
use std::net::SocketAddr;

use crate::proxy::config::Config;
use crate::proxy::connection::{NodeConnection, TargetConnection, new_tunnel_id};
use crate::proxy::target::{Target, TargetDumpOrder, calc_target_id_by_endpoint, dump_targets};
use crate::proxy::config::{read_config};
use crate::proxy::g::{SERVER_INFO,NODE_LOCAL_SELECTOR};
use std::ops::Deref;
use log::{error, info};


#[derive(Debug)]
pub struct ProxyServer {
    pub server_config: Config,
    pub targets_info: Arc<tokio::sync::Mutex<HashMap<String, Target>>>,
    pub tunnel_info: Arc<tokio::sync::Mutex<HashMap<String, (NodeConnection, TargetConnection)>>>,
}

impl ProxyServer {
    pub fn new() -> ProxyServer{
        ProxyServer {
            server_config: read_config(),
            targets_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            tunnel_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

pub async fn connect_to_target_with_least_conn() -> (Option<tokio::net::TcpStream>, Option<Target>) {
    let targets_dump = dump_targets(TargetDumpOrder::AscOrder).await;

    let mut tcp_stream_target: Option<tokio::net::TcpStream> = None;
    let mut conn_target_info: Option<Target> = None;

    // try to connect to the target with the least connection
    for t in targets_dump.iter() {
        if !t.target.target_active {
            continue;
        }
        if t.target_conn_count > t.target.target_max_conn {
            continue;
        }

        let r = tokio::net::TcpSocket::new_v4();
        let socket_conn = match r {
            Ok(s) => {
                if SERVER_INFO.deref().server_config.lb_node.enable_local_endpoints &&
                    SERVER_INFO.deref().server_config.lb_node.local_endpoints.len() > 0 {
                    let u = NODE_LOCAL_SELECTOR.deref().load(Ordering::Relaxed);
                    NODE_LOCAL_SELECTOR.deref().store(u+1, Ordering::Relaxed);

                    let local_socket_addr : SocketAddr = SERVER_INFO.deref().server_config
                        .lb_node.local_endpoints[(u as usize % (SERVER_INFO.deref().server_config.lb_node.local_endpoints.len()))].parse()
                        .expect(&*format!("Invalid node local endpoint [{}]",
                                          SERVER_INFO.deref().server_config.lb_node
                                              .local_endpoints[(u as usize % (SERVER_INFO.deref().server_config.lb_node.local_endpoints.len()))]));

                    s.bind(local_socket_addr)
                        .expect(&*format!("Bind node local endpoint [{}] fail",
                                          SERVER_INFO.deref().server_config.lb_node
                                              .local_endpoints[(u as usize % (SERVER_INFO.deref().server_config.lb_node.local_endpoints.len()))]));
                }
                Some(s)
            },
            Err(_) => continue
        };
        let connect_timeout = tokio::time::Duration::from_secs(5);
        if let Ok(r) = tokio::time::timeout(connect_timeout,
                                            socket_conn.unwrap().connect(t.target.target_endpoint.parse().unwrap())).await {
            tcp_stream_target = match r {
                Ok(c) => Some(c),
                Err(_) => continue
            };

            conn_target_info = Some(t.target.clone());
            break;
        } else {
            continue;
        }
    }

    return (tcp_stream_target, conn_target_info)
}

pub async fn start_tcp_proxy_server() -> Result<(), Box<dyn Error>>{
    let node_listener = tokio::net::TcpListener::bind(SERVER_INFO.deref().server_config.lb_node.listen.as_str())
        .await.expect(format!("Failure binding node listen endpoint [{}]", SERVER_INFO.deref().server_config.lb_node.listen).as_str());

    loop {
        let (mut tcp_stream_node, node_remote_addr) = node_listener.accept().await?;
        info!("remote connection from {}", node_remote_addr);

        let node_connection_count = SERVER_INFO.deref().tunnel_info.lock().await.len() as u32;
        if node_connection_count > SERVER_INFO.deref().server_config.lb_node.max_conn {
            let _ = tcp_stream_node.shutdown().await;
            continue;
        }

        let (tcp_stream_target, conn_target_info) = connect_to_target_with_least_conn().await;

        match tcp_stream_target {
            Some(_) => (),
            None => {
                let _ = tcp_stream_node.shutdown().await;
                continue;
            }
        }

        let node_timeout = SERVER_INFO.deref().server_config.lb_node.timeout.clone();
        let target_timeout = conn_target_info.clone().unwrap().target_timeout.clone();

        let conn_target_id = calc_target_id_by_endpoint(conn_target_info.clone().unwrap().target_endpoint);
        let tcp_stream_target = tcp_stream_target.unwrap();
        let target_local_addr = tcp_stream_target.local_addr().unwrap().to_string();

        let (mut tcp_stream_node_read, mut tcp_stream_node_write) = tcp_stream_node.into_split();
        let (mut tcp_stream_target_read, mut tcp_stream_target_write) = tcp_stream_target.into_split();

        let node_connection_info = NodeConnection::new(
            SERVER_INFO.deref().server_config.lb_node.listen.clone(),
            node_remote_addr.to_string());

        let target_connection_info = TargetConnection::new(
            target_local_addr.clone(),
            conn_target_info.clone().unwrap().target_endpoint,
            conn_target_id
        );

        let tunnel_id = new_tunnel_id();
        let tunnel_info_arc = Arc::clone(&SERVER_INFO.deref().tunnel_info);
        let tunnel_id_dump = tunnel_id.clone();
        let tunnel_info_arc_dump = Arc::clone(&tunnel_info_arc);

        SERVER_INFO.deref().tunnel_info.lock().await.insert(tunnel_id.clone(), (node_connection_info, target_connection_info));
        info!("build tunnel |{}| successfully, node: {}->{}, target: {}->{}",
                 tunnel_id,
                 node_remote_addr.to_string(),
                 SERVER_INFO.deref().server_config.lb_node.listen.clone(),
                 target_local_addr.clone(),
                 conn_target_info.clone().unwrap().target_endpoint);

        // task of reading from node connection and then writing to target connection
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let mut count;
            loop {
                let read_timeout = tokio::time::Duration::from_secs(node_timeout as u64);
                if let Ok(r) = tokio::time::timeout(read_timeout, tcp_stream_node_read.read(&mut buf)).await {
                    match r {
                        Ok(n) if n == 0 => {
                            tunnel_info_arc.lock().await.remove(&tunnel_id);
                            info!("|{}| tcp_stream_node_read: closed by remote", tunnel_id);
                            return;
                        },
                        Ok(n) => {
                            count = n;
                            let tunnel_info = tunnel_info_arc.lock().await;
                            let v = tunnel_info.get(&tunnel_id);
                            match v {
                                Some((node_info, target_info)) => {
                                    let mut node_info_dump = node_info.clone();
                                    let target_info_dump = target_info.clone();
                                    node_info_dump.add_read_n(n as u64);
                                    drop(tunnel_info);
                                    tunnel_info_arc.lock().await.insert(tunnel_id.clone(), (node_info_dump, target_info_dump));
                                },
                                None => {
                                    drop(tunnel_info);
                                    tunnel_info_arc.lock().await.remove(&tunnel_id);
                                    error!("|{}| tcp_stream_node_read: not find, disconnect", tunnel_id);
                                    return;
                                },
                            }
                        },
                        Err(e) => {
                            tunnel_info_arc.lock().await.remove(&tunnel_id.clone());
                            error!("|{}| tcp_stream_node_read: failed to read from socket; err = {:?}", tunnel_id, e);
                            return;
                        }
                    };
                } else {
                    // read from node timeout
                    tunnel_info_arc.lock().await.remove(&tunnel_id);
                    error!("|{}| tcp_stream_node_read: timeout", tunnel_id);
                    return;
                }

                let write_timeout = tokio::time::Duration::from_secs(target_timeout as u64);
                if let Ok(r) = tokio::time::timeout(write_timeout, tcp_stream_target_write.write_all(&buf[0..count])).await {
                    match r {
                        Ok(_) => {
                            let tunnel_info = tunnel_info_arc.lock().await;
                            let v = tunnel_info.get(&tunnel_id);
                            match v {
                                Some((node_info, target_info)) => {
                                    let node_info_dump = node_info.clone();
                                    let mut target_info_dump = target_info.clone();
                                    target_info_dump.add_write_n(count as u64);
                                    drop(tunnel_info);
                                    tunnel_info_arc.lock().await.insert(tunnel_id.clone(), (node_info_dump, target_info_dump));
                                },
                                None => {
                                    drop(tunnel_info);
                                    tunnel_info_arc.lock().await.remove(&tunnel_id);
                                    error!("|{}| tcp_stream_target_write: not find, disconnect", tunnel_id);
                                    return;
                                },
                            }
                        }
                        Err(e) => {
                            tunnel_info_arc.lock().await.remove(&tunnel_id);
                            error!("|{}| tcp_stream_target_write: failed to write to socket; err = {:?}", tunnel_id, e);
                            return;
                        }
                    }
                } else {
                    // write to target timeout
                    tunnel_info_arc.lock().await.remove(&tunnel_id);
                    error!("|{}| tcp_stream_target_write: timeout", tunnel_id);
                    return;
                }
            }
        });

        // task of reading from target connection and then writing to node connection
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let mut count;
            loop {
                let read_timeout = tokio::time::Duration::from_secs(target_timeout as u64);
                if let Ok(r) = tokio::time::timeout(read_timeout, tcp_stream_target_read.read(&mut buf)).await {
                    match r {
                        Ok(n) if n == 0 => {
                            tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                            info!("|{}| tcp_stream_target_read: closed by remote", tunnel_id_dump);
                            return
                        },
                        Ok(n) => {
                            count = n;
                            let tunnel_info = tunnel_info_arc_dump.lock().await;
                            let v = tunnel_info.get(&tunnel_id_dump);
                            match v {
                                Some((node_info, target_info)) => {
                                    let node_info_dump = node_info.clone();
                                    let mut target_info_dump = target_info.clone();
                                    target_info_dump.add_read_n(n as u64);
                                    drop(tunnel_info);
                                    tunnel_info_arc_dump.lock().await.insert(tunnel_id_dump.clone(), (node_info_dump, target_info_dump));
                                },
                                None => {
                                    drop(tunnel_info);
                                    tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                                    error!("|{}| tcp_stream_target_read: not find, disconnect", tunnel_id_dump);
                                    return
                                },
                            }
                        },
                        Err(e) => {
                            tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                            error!("|{}| tcp_stream_target_read: failed to read from socket; err = {:?}", tunnel_id_dump, e);
                            return;
                        }
                    };
                } else {
                    // read from target timeout
                    tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                    error!("|{}| tcp_stream_target_read: timeout", tunnel_id_dump);
                    return;
                }

                let write_timeout = tokio::time::Duration::from_secs(node_timeout as u64);
                if let Ok(r) = tokio::time::timeout(write_timeout, tcp_stream_node_write.write_all(&buf[0..count])).await {
                    match r {
                        Ok(_) => {
                            {
                                let tunnel_info = tunnel_info_arc_dump.lock().await;
                                let v = tunnel_info.get(&tunnel_id_dump);
                                match v {
                                    Some((node_info, target_info)) => {
                                        let mut node_info_dump = node_info.clone();
                                        let target_info_dump = target_info.clone();
                                        node_info_dump.add_write_n(count as u64);
                                        drop(tunnel_info);
                                        tunnel_info_arc_dump.lock().await.insert(tunnel_id_dump.clone(), (node_info_dump, target_info_dump));
                                    },
                                    None => {
                                        drop(tunnel_info);
                                        tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                                        error!("|{}| tcp_stream_node_write: not find, disconnect", tunnel_id_dump);
                                        return;
                                    },
                                }
                            }
                        }
                        Err(e) => {
                            tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                            error!("|{}| tcp_stream_node_write: failed to write to socket; err = {:?}", tunnel_id_dump, e);
                            return;
                        }
                    }
                } else {
                    // write to node timeout
                    tunnel_info_arc_dump.lock().await.remove(&tunnel_id_dump);
                    error!("|{}| tcp_stream_node_write: timeout", tunnel_id_dump);
                    return;
                }
            }
        });
    }
}
