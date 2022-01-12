use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::SinkExt;

use tokio::net::{TcpStream, TcpListener};
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};
use tokio_stream::StreamExt;

/// Represents messages that are passed by thread actors in shared mpsc
#[derive(Clone)]
struct Message {
    payload: String,
    from: SocketAddr,
}

/// Handy typedefs
type Tx = mpsc::UnboundedSender<Message>;
type Rx = mpsc::UnboundedReceiver<Message>;

/// This is the main shared state that owned handles of are cloned and
/// passed around to each client, so they can easily call the broadcast method below, or view
/// all connected clients
type ClientState = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

/// State for the server
pub struct Server {
    socket: SocketAddr,
    clients: ClientState,
}

/// Client state
pub struct Client {
    /// main rx & tx along TCP stream 
    lines: Framed<TcpStream, LinesCodec>,
    /// recieve half of mpsc channel, used to recieve messages
    /// from other clients
    rx: Rx,
    /// connecting address of client
    addr: SocketAddr,
    /// registered username to display to other clients
    username: String,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Server {
            socket: addr.parse::<SocketAddr>().unwrap(),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(self.socket).await.unwrap();

        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            println!("new client connected: {:?}", addr);
            let clients = Arc::clone(&self.clients);
            tokio::spawn(async move {
                match handle_client(clients, socket, addr).await {
                    Ok(_) => {
                        // do nothing
                    },
                    Err(e) => println!("error occured while processing client {:?}", e),
                }
            });
        }
    }

}

/// Main process loop that handles each client. This will loop indefinitely until the client disconnects
///
/// Process:
///     - init client with username
///     - broadcoast client connection to all users
///     - loop:
///         - if received msg from mpsc, tx across tcp to client
///         - else if rx across tcp from client, broadcast to other clients
///         - else, nothing happened so we loop again
async fn handle_client(clients: ClientState, socket: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {

    let mut client = Client::new(clients.clone(), socket, addr).await.unwrap();
    client.lines.send("enter your username: ").await?;
    client.username = match client.lines.next().await {
        Some(Ok(line)) => line,
        // We didn't get a line so we return early here.
        _ => {
            return Ok(());
        }
    };
    println!("client registered: {}", client.username);

    // broadcast once that this client entered the chatroom
    {
        let msg = Message {
            payload: format!("{} has entered the chatroom!", client.username),
            from: client.addr,
        };
        broadcast(Arc::clone(&clients), msg).await;
    }

    // main process loop
    loop {
        tokio::select! {
            // message recieved from other client
            Some(msg) = client.rx.recv() => client.lines.send(msg.payload.clone()).await?,
            tx = client.lines.next() => match tx {
                Some(Ok(tx)) => {
                    let msg = Message {
                        payload: format!("[{}]: {}", client.username, tx),
                        from: client.addr,
                    };
                    broadcast(Arc::clone(&clients), msg).await;
                },
                Some(_) => {},
                None => break,
            }
        }
    }

    Ok(())
}

/// Broadcasts to every client except the sender of the message
async fn broadcast(clients: ClientState, message: Message) {
    for client in clients.lock().await.iter_mut() {
        if *client.0 != message.from {
            let _ = client.1.send(message.clone());
        }
    }
}

impl Client {
    /// Creates a new client
    async fn new(
        clients: ClientState,
        socket: TcpStream,
        addr: SocketAddr,
        ) -> Result<Self, Box<dyn Error>> {
        let lines = Framed::new(socket, LinesCodec::new());
        let (tx, rx) = mpsc::unbounded_channel();
        clients.lock().await.insert(addr, tx);
        Ok( Client {
            lines,
            rx,
            addr,
            username: "anon".to_string(),
        })
    }
}
