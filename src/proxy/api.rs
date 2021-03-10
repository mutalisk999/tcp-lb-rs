use std::error::Error;

// use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use crate::proxy::proxy::{ProxyServer};
use std::sync::Arc;
use std::collections::HashMap;
use crate::proxy::target::Target;
use crate::proxy::connection::{NodeConnection, TargetConnection};


async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive as we did with the above uppercase endpoint.
        // So here we do `.await` on the future, waiting on concatenating the full body,
        // then afterwards the content can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


pub async fn start_api_server(proxy_server: &ProxyServer
) -> Result<(), Box<dyn Error>>{
    let addr = proxy_server.server_config.lb_api.listen.clone().parse()?;
    let service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(
            move |req| {
                echo(req)
            }))
    });
    let server = Server::bind(&addr).serve(service);
    server.await?;
    Ok(())
}