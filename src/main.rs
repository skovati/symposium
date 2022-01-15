use std::env;
use std::net::SocketAddr;

mod user;
mod router;
use router::*;

#[tokio::main]
async fn main() {
    // parse command line args
    let socket: SocketAddr = if env::args().len() > 2 {
        let args: Vec<String> = env::args().collect();
        let ip = &args[1];
        let port = &args[2];
        format!("{}:{}", ip, port)
            .parse()
            // default 
            .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)))
    } else {
        SocketAddr::from(([127, 0, 0, 1], 8080))
    };

    // create and run router
    let router = Router::new(socket);
    router.run().await;
}
