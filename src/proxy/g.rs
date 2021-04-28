use crate::proxy::proxy::ProxyServer;
use std::sync::atomic::{AtomicU64};

lazy_static! {
    pub static ref SERVER_INFO: ProxyServer = ProxyServer::new();
    pub static ref NODE_LOCAL_SELECTOR: AtomicU64 = AtomicU64::new(0);
}