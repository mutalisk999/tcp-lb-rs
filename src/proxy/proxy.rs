use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

use crate::proxy::config::Config;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::proxy::connection::{NodeConnection, TargetConnection};
use crate::proxy::target::Target;

pub async fn start_tcp_proxy(config: &Config, targets: Arc<Mutex<HashMap<String, Target>>>,
                             conn_pair_n2t: Arc<Mutex<HashMap<Arc<Mutex<NodeConnection>>, Arc<Mutex<TargetConnection>>>>>,
                             conn_pair_t2n: Arc<Mutex<HashMap<Arc<Mutex<TargetConnection>>, Arc<Mutex<NodeConnection>>>>>
    ) -> Result<(), Box<dyn Error>>{

    let listener = TcpListener::bind(config.lb_node.listen.as_str())
        .await.expect(format!("Failure binding node listen endpoint [{}]", config.lb_node.listen).as_str());

    loop {
        let (mut socket, remote_addr) = listener.accept().await?;
        let conn = Arc::new(Mutex::new(NodeConnection::new(socket)));
        println!("remote connection from {}", remote_addr);



        tokio::spawn(async move {

            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match conn.lock().await.connection.tcp_stream.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = conn.lock().await.connection.tcp_stream.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
