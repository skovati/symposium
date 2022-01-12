mod server;
use self::server::Server;
use std::env;

#[tokio::main]
async fn main() {
    let addr = env::args()
            .nth(1)
            .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let server = Server::new(&addr);
    println!("starting server on port {}", addr);
    server.run().await;
}
