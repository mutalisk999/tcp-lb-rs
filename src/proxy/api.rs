// #[macro_use]
use serde::{Serialize, Deserialize};

use std::error::Error;

// use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use crate::proxy::g::SERVER_INFO;
use std::ops::Deref;
use std::collections::HashMap;
use url::form_urlencoded;
use crate::proxy::connection::get_target_conn_count_by_target_id;


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
    pub fn new(_target_id: String, _endpoint: String, _max_conn: u32, _timeout: u32, _conn_count: u32, _active: bool) -> TargetInfoResp {
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

async fn request_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "<h1>Hello World</h1>",
        ))),

        (&Method::GET, "/api/get_node_info") => {
            let node_info_resp = NodeInfoResp::new(
                SERVER_INFO.deref().server_config.lb_node.listen.clone(),
                SERVER_INFO.deref().server_config.lb_node.max_conn.clone(),
                SERVER_INFO.deref().server_config.lb_node.timeout.clone(),
                SERVER_INFO.deref().tunnel_info.lock().await.len() as u32
            );
            let json_resp = JsonResp::new(1, node_info_resp, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        },

        (&Method::POST, "/api/get_targets_info") => {
            let mut targets_info_resp = vec![];
            for (k, target) in SERVER_INFO.deref().targets_info.lock().await.iter() {
                let target_info_resp = TargetInfoResp::new(
                    k.clone(),
                    target.target_endpoint.clone(),
                    target.target_max_conn.clone(),
                    target.target_timeout.clone(),
                    get_target_conn_count_by_target_id(k.clone()).await,
                    target.target_active.clone()
                );
                targets_info_resp.push(target_info_resp.clone());
            }
            let json_resp = JsonResp::new(1, targets_info_resp, None);
            let ret_str = serde_json::to_string(&json_resp).unwrap();
            Ok(Response::new(Body::from(ret_str)))
        },

        (&Method::GET, "/api/get_target_tunnel_info") => {
            let b = hyper::body::to_bytes(req).await?;
            let params = form_urlencoded::parse(b.as_ref())
                .into_owned()
                .collect::<HashMap<String, String>>();
            let target_id = if let Some(target_id) = params.get("target_id") {
                target_id
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body("Missing field".into())
                    .unwrap());
            };

            let ret_str = target_id.clone();
            Ok(Response::new(Body::from(ret_str)))
        },

        (&Method::GET, "/api/get_tunnel_info") => {
            let ret_str = "";
            Ok(Response::new(Body::from(ret_str)))
        },

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn start_api_server() -> Result<(), Box<dyn Error>>{
    let addr = SERVER_INFO.deref().server_config.lb_api.listen.clone().parse()?;
    let service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(request_handler))
    });
    let server = Server::bind(&addr).serve(service);
    server.await?;
    Ok(())
}