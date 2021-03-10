use crate::proxy::proxy::ProxyServer;

lazy_static! {
    pub static ref SERVER_INFO: ProxyServer = ProxyServer::new();
}