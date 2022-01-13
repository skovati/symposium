use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use chrono;

/// Handy typedefs
type Tx = mpsc::UnboundedSender<Message>;
type Rx = UnboundedReceiverStream<Message>;

#[derive(Debug, Clone)]
struct User {
    name: String,
    id: Uuid,
}

/// this is the mutable state passed around by thread actors
/// the HashMap enclosed is wrapped in a Mutex and Atomic ref count in order
/// to be thread safe
#[derive(Clone)]
struct SharedState {
    users: Arc<Mutex<HashMap<String, Tx>>>,
    connected: Arc<Mutex<usize>>,
}

impl SharedState {
    fn new() -> Self {
        SharedState {
            users: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(Mutex::new(0)),
        }
    }

    async fn broadcast(&self, name: String, msg: Message) {
        // Skip any non-Text messages...
        let msg = if let Ok(s) = msg.to_str() {
            s
        } else {
            return;
        };

        let now = chrono::offset::Local::now();
        let new_msg = format!("[{}] {}: {}", now.format("%I:%M"), name, msg);

        // New message from this user, send it to everyone else (except same uid)...
        for (_, tx) in self.users.lock().await.iter() {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // tx disconnected, so disconnect_user will run in the other task
            }
        }
    }

    async fn disconnect_user(&self, name: String) {
        eprintln!("good bye user: {}", name);

        // Stream closed up, so remove from the user list
        self.users.lock().await.remove(&name);
    }
}

#[tokio::main]
async fn main() {
    let state = SharedState::new();
    let state = warp::any().map(move || state.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("ws")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(state)
        .map(|ws: warp::ws::Ws, state| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| handle_user(socket, state))
        });

    // GET / serves static index.html, app.js
    let index = warp::path::end()
        .and(warp::fs::dir("www"));

    let stat = warp::path("static")
        .and(warp::fs::dir("www/static"));

    let routes = index.or(stat).or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_user(ws: WebSocket, state: SharedState) {
    // split into tx and rx halves of the websocket
    let (mut ws_tx, mut ws_rx) = ws.split();

    // unbounded channel handles buffering and flushing
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    let id: Uuid = Uuid::new_v4();

    // get username
    let username: String;
    ws_tx.send(Message::text("Welcome, enter a username below to get started!")).await.unwrap();
    if let Some(response) = ws_rx.next().await {
        username = match response {
            Ok(name) => name.to_str().unwrap_or("anon").to_string(),
            Err(e) => {
                eprintln!("websocket error(id={}): {}", id, e);
                return
            }
        };
    } else {
        return
    }
    println!("GOT HERE FOR {}", username);


    let user = User {
        name: username.clone(),
        id,
    };

    let msg: String;
    {
        let mut connected = state.connected.lock().await;
        *connected += 1;
        msg = format!("{} just joined the room! There are now {} user(s) connected.", user.name, *connected);
    }

    {
        // register this user in the server state
        state.users.lock().await.insert(user.name.clone(), tx);
    }

    state.broadcast("ANOUNCE".to_string(), Message::text(msg)).await;

    println!("new user connected: {:?}", user);

    // spawn a task to handle writing messages to the websocket that
    // are received from other users on the mpsc
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            println!("RECEIVED {:?}", message);
            ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // when we recieve text on the websocket from the frontend,
    // broadcast it to all connected users on the mpsc
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(id={}): {}", user.id, e);
                break;
            }
        };
        state.broadcast(user.name.clone(), msg).await;
    }

    // the while loop will continue as long as the websocket is open
    // when it closes, we finally reach this disconnect function call
    state.disconnect_user(user.name).await;
}
