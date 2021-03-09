use std::error::Error;

use crate::proxy::proxy::{ProxyServer};


pub async fn start_api_server(proxy_server: &ProxyServer
) -> Result<(), Box<dyn Error>>{
    let _ = proxy_server;
    Ok(())
}