extern crate tokio;

mod proxy;
use proxy::proxy::{start_tcp_proxy, start_tcp_proxy2};

#[tokio::main]
async fn main() {
    let f = start_tcp_proxy();

    let f2 = start_tcp_proxy2();

    let (_, _) = tokio::join!(f, f2);
}