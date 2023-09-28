// #[macro_use]
use serde::{Deserialize, Serialize};

use std::error::Error;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use crate::proxy::connection::get_target_conn_count_by_target_id;
use crate::proxy::g::SERVER_INFO;
use chrono::Utc;
use std::collections::HashMap;
use std::ops::Deref;
use url::form_urlencoded;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JsonResp<T> {
    pub id: u64,
    pub result: T,
    pub error: Option<String>,
}

impl<T> JsonResp<T> {
    pub fn new(_id: u64, _result: T, _error: Option<String>) -> JsonResp<T> {
        JsonResp {
            id: _id,
            result: _result,
            error: _error,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NodeInfoResp {
    pub listen: String,
    pub max_conn: u32,
    pub timeout: u32,
    pub conn_count: u32,
}

impl NodeInfoResp {
    pub fn new(_listen: String, _max_conn: u32, _timeout: u32, _conn_count: u32) -> NodeInfoResp {
        NodeInfoResp {
            listen: _listen,
            max_conn: _max_conn,
            timeout: _timeout,
            conn_count: _conn_count,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TargetInfoResp {
    pub target_id: String,
    pub endpoint: String,
    pub max_conn: u32,
    pub timeout: u32,
    pub conn_count: u32,
    pub active: bool,
}

impl TargetInfoResp {
    pub fn new(
        _target_id: String,
        _endpoint: String,
        _max_conn: u32,
        _timeout: u32,
        _conn_count: u32,
        _active: bool,
    ) -> TargetInfoResp {
        TargetInfoResp {
            target_id: _target_id,
            endpoint: _endpoint,
            max_conn: _max_conn,
            timeout: _timeout,
            conn_count: _conn_count,
            active: _active,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NodeConnectionInfoResp {
    pub connect_id: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub create_time: i64,
    pub read_speed_1m: u64,
    pub read_speed_5m: u64,
    pub read_speed_30m: u64,
    pub write_speed_1m: u64,
    pub write_speed_5m: u64,
    pub write_speed_30m: u64,
}

impl NodeConnectionInfoResp {
    pub fn new(
        _connect_id: String,
        _local_endpoint: String,
        _remote_endpoint: String,
        _create_time: i64,
        _read_speed_1m: u64,
        _read_speed_5m: u64,
        _read_speed_30m: u64,
        _write_speed_1m: u64,
        _write_speed_5m: u64,
        _write_speed_30m: u64,
    ) -> NodeConnectionInfoResp {
        NodeConnectionInfoResp {
            connect_id: _connect_id,
            local_endpoint: _local_endpoint,
            remote_endpoint: _remote_endpoint,
            create_time: _create_time,
            read_speed_1m: _read_speed_1m,
            read_speed_5m: _read_speed_5m,
            read_speed_30m: _read_speed_30m,
            write_speed_1m: _write_speed_1m,
            write_speed_5m: _write_speed_5m,
            write_speed_30m: _write_speed_30m,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TargetConnectionInfoResp {
    pub connect_id: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub create_time: i64,
    pub read_speed_1m: u64,
    pub read_speed_5m: u64,
    pub read_speed_30m: u64,
    pub write_speed_1m: u64,
    pub write_speed_5m: u64,
    pub write_speed_30m: u64,
    pub target_id: String,
}

impl TargetConnectionInfoResp {
    pub fn new(
        _connect_id: String,
        _local_endpoint: String,
        _remote_endpoint: String,
        _create_time: i64,
        _read_speed_1m: u64,
        _read_speed_5m: u64,
        _read_speed_30m: u64,
        _write_speed_1m: u64,
        _write_speed_5m: u64,
        _write_speed_30m: u64,
        _target_id: String,
    ) -> TargetConnectionInfoResp {
        TargetConnectionInfoResp {
            connect_id: _connect_id,
            local_endpoint: _local_endpoint,
            remote_endpoint: _remote_endpoint,
            create_time: _create_time,
            read_speed_1m: _read_speed_1m,
            read_speed_5m: _read_speed_5m,
            read_speed_30m: _read_speed_30m,
            write_speed_1m: _write_speed_1m,
            write_speed_5m: _write_speed_5m,
            write_speed_30m: _write_speed_30m,
            target_id: _target_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TunnelInfoResp {
    pub tunnel_id: String,
    pub node_connection: NodeConnectionInfoResp,
    pub target_connection: TargetConnectionInfoResp,
}

impl TunnelInfoResp {
    pub fn new(
        _tunnel_id: String,
        _node_connection: NodeConnectionInfoResp,
        _target_connection: TargetConnectionInfoResp,
    ) -> TunnelInfoResp {
        TunnelInfoResp {
            tunnel_id: _tunnel_id,
            node_connection: _node_connection,
            target_connection: _target_connection,
        }
    }
}

async fn request_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") | (&Method::POST, "/") => {
            Ok(Response::new(Body::from("<h1>Hello World</h1>")))
        }

        (&Method::GET, "/api/get_node_info") | (&Method::POST, "/api/get_node_info") => {
            let node_info_resp = NodeInfoResp::new(
                SERVER_INFO.deref().server_config.lb_node.listen.clone(),
                SERVER_INFO.deref().server_config.lb_node.max_conn.clone(),
                SERVER_INFO.deref().server_config.lb_node.timeout.clone(),
                SERVER_INFO.deref().tunnel_info.lock().await.len() as u32,
            );
            let json_resp = JsonResp::new(1, node_info_resp, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        }

        (&Method::GET, "/api/get_targets_info") | (&Method::POST, "/api/get_targets_info") => {
            let mut targets_info_resp = vec![];
            for (k, target) in SERVER_INFO.deref().targets_info.lock().await.iter() {
                let target_info_resp = TargetInfoResp::new(
                    k.clone(),
                    target.target_endpoint.clone(),
                    target.target_max_conn.clone(),
                    target.target_timeout.clone(),
                    get_target_conn_count_by_target_id(k.clone()).await,
                    target.target_active.clone(),
                );
                targets_info_resp.push(target_info_resp.clone());
            }
            let json_resp = JsonResp::new(1, targets_info_resp, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        }

        (&Method::GET, "/api/get_target_tunnel_info")
        | (&Method::POST, "/api/get_target_tunnel_info") => {
            // try to parse query string params from url
            let mut params = req
                .uri()
                .query()
                .map(|v| {
                    url::form_urlencoded::parse(v.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);

            if params.len() == 0 {
                // try to parse query string params from body
                let b = hyper::body::to_bytes(req).await?;
                params = form_urlencoded::parse(b.as_ref())
                    .into_owned()
                    .collect::<HashMap<String, String>>();
            }

            let target_id = if let Some(target_id) = params.get("target_id") {
                target_id
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body("Missing field".into())
                    .unwrap());
            };

            let mut target_tunnel_info = vec![];
            for (k, v) in SERVER_INFO.deref().tunnel_info.lock().await.iter() {
                if target_id.deref() == v.1.target_id {
                    let (node_connection, target_connection) = v.clone();
                    let node_connection_resp = NodeConnectionInfoResp::new(
                        node_connection.connection.connect_id.clone(),
                        node_connection.connection.local_endpoint.clone(),
                        node_connection.connection.remote_endpoint.clone(),
                        node_connection.connection.create_time.clone(),
                        node_connection.connection.read_bytes_1m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_1m)
                                as u64,
                        node_connection.connection.read_bytes_5m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_5m)
                                as u64,
                        node_connection.connection.read_bytes_30m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_30m)
                                as u64,
                        node_connection.connection.write_bytes_1m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_1m)
                                as u64,
                        node_connection.connection.write_bytes_5m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_5m)
                                as u64,
                        node_connection.connection.write_bytes_30m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - node_connection.connection.start_time_30m)
                                as u64,
                    );
                    let target_connection_resp = TargetConnectionInfoResp::new(
                        target_connection.connection.connect_id.clone(),
                        target_connection.connection.local_endpoint.clone(),
                        target_connection.connection.remote_endpoint.clone(),
                        target_connection.connection.create_time.clone(),
                        target_connection.connection.read_bytes_1m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_1m)
                                as u64,
                        target_connection.connection.read_bytes_5m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_5m)
                                as u64,
                        target_connection.connection.read_bytes_30m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_30m)
                                as u64,
                        target_connection.connection.write_bytes_1m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_1m)
                                as u64,
                        target_connection.connection.write_bytes_5m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_5m)
                                as u64,
                        target_connection.connection.write_bytes_30m * 8 as u64 * 1000000000 as u64
                            / (Utc::now().timestamp_nanos()
                                - target_connection.connection.start_time_30m)
                                as u64,
                        v.1.target_id.clone(),
                    );
                    let tunnel_info_resp = TunnelInfoResp::new(
                        k.clone(),
                        node_connection_resp,
                        target_connection_resp,
                    );
                    target_tunnel_info.push(tunnel_info_resp);
                }
            }

            let json_resp = JsonResp::new(1, target_tunnel_info, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        }

        (&Method::GET, "/api/get_tunnel_info") | (&Method::POST, "/api/get_tunnel_info") => {
            let mut target_tunnel_info = vec![];
            for (k, v) in SERVER_INFO.deref().tunnel_info.lock().await.iter() {
                let (node_connection, target_connection) = v.clone();
                let node_connection_resp = NodeConnectionInfoResp::new(
                    node_connection.connection.connect_id.clone(),
                    node_connection.connection.local_endpoint.clone(),
                    node_connection.connection.remote_endpoint.clone(),
                    node_connection.connection.create_time.clone(),
                    node_connection.connection.read_bytes_1m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_1m)
                            as u64,
                    node_connection.connection.read_bytes_5m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_5m)
                            as u64,
                    node_connection.connection.read_bytes_30m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_30m)
                            as u64,
                    node_connection.connection.write_bytes_1m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_1m)
                            as u64,
                    node_connection.connection.write_bytes_5m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_5m)
                            as u64,
                    node_connection.connection.write_bytes_30m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos() - node_connection.connection.start_time_30m)
                            as u64,
                );
                let target_connection_resp = TargetConnectionInfoResp::new(
                    target_connection.connection.connect_id.clone(),
                    target_connection.connection.local_endpoint.clone(),
                    target_connection.connection.remote_endpoint.clone(),
                    target_connection.connection.create_time.clone(),
                    target_connection.connection.read_bytes_1m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_1m)
                            as u64,
                    target_connection.connection.read_bytes_5m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_5m)
                            as u64,
                    target_connection.connection.read_bytes_30m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_30m)
                            as u64,
                    target_connection.connection.write_bytes_1m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_1m)
                            as u64,
                    target_connection.connection.write_bytes_5m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_5m)
                            as u64,
                    target_connection.connection.write_bytes_30m * 8 as u64 * 1000000000 as u64
                        / (Utc::now().timestamp_nanos()
                            - target_connection.connection.start_time_30m)
                            as u64,
                    v.1.target_id.clone(),
                );
                let tunnel_info_resp =
                    TunnelInfoResp::new(k.clone(), node_connection_resp, target_connection_resp);
                target_tunnel_info.push(tunnel_info_resp);
            }

            let json_resp = JsonResp::new(1, target_tunnel_info, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn start_api_server() -> Result<(), Box<dyn Error>> {
    let addr = SERVER_INFO
        .deref()
        .server_config
        .lb_api
        .listen
        .clone()
        .parse()?;
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(request_handler)) });
    let server = Server::bind(&addr).serve(service);
    server.await?;
    Ok(())
}
