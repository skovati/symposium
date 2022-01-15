// 3rd party imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use serde_json;

// custom crates
use crate::user::*;
use crate::parcel::Parcel;

/// this is the mutable state passed around by thread actors
/// the HashMap enclosed is wrapped in a Mutex and Atomic ref count in order
/// to be thread safe
#[derive(Clone)]
pub struct Router {
    users: Arc<Mutex<HashMap<String, Tx>>>,
    addr: SocketAddr,
    connected: Arc<Mutex<usize>>,
    admin: User,
}

impl Router {
    pub fn new(addr: SocketAddr) -> Self {
        Router {
            users: Arc::new(Mutex::new(HashMap::new())),
            addr,
            connected: Arc::new(Mutex::new(0)),
            admin: User {
                name: "server".to_string(),
            }
        }
    }

    pub async fn run(&self) {
        let router = self.clone();
        let router = warp::any().map(move || router.clone());

        // GET /ws -> websocket upgrade
        let chat = warp::path("ws")
            // The `ws()` filter will prepare Websocket handshake...
            .and(warp::ws())
            .and(router)
            .map(|ws: warp::ws::Ws, router: Router| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| router.handle_user(socket))
            });

        // GET / -> serves static index.html, app.js
        let files = warp::fs::dir("static");

        let routes = files.or(chat);
        println!("server started at: {}", self.addr);
        warp::serve(routes).run(self.addr).await;
    }

    async fn broadcast(&self, msg: &String, from: &User) {
        let parcel = Parcel::new(&msg, &from);
        let new_msg = serde_json::to_string(&parcel).unwrap();

        // New message from this user, send it to everyone else (except same uid)...
        for (_, tx) in self.users.lock().await.iter() {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // tx disconnected, so disconnect_user will run in the other task
            }
        }
    }

    // async fn target_msg(&self, msg: &String, to: &User) {
    //     let parcel = Parcel::new(&msg, &self.admin);
    //     let new_msg = serde_json::to_string(&parcel).unwrap();
    //
    //     // New message from this user, send it to everyone else (except same uid)...
    //     for (_, tx) in self.users.lock().await.iter() {
    //         if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
    //             // tx disconnected, so disconnect_user will run in the other task
    //         }
    //     }
    // }

    async fn disconnect_user(&self, name: String) {
        println!("disconnected user: {}", name);

        let msg: String;
        {
            let mut connected = self.connected.lock().await;
            *connected -= 1;
            msg = format!("{} just left the room. There are now {} user(s) connected.", name, *connected);
        }

        self.broadcast(&msg, &self.admin).await;

        // Stream closed up, so remove from the user list
        self.users.lock().await.remove(&name);
    }

    async fn handle_user(self, ws: WebSocket) {
        // split into tx and rx halves of the websocket
        let (mut ws_tx, mut ws_rx) = ws.split();

        // unbounded channel handles buffering and flushing
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        // get username
        let username: String;
        let parcel = Parcel::new(&"Welcome, enter a username below to get started!".to_string(), &self.admin);
        let parcel_json = serde_json::to_string(&parcel).unwrap();
        ws_tx.send(Message::text(parcel_json)).await.unwrap();
        if let Some(response) = ws_rx.next().await {
            username = match response {
                Ok(name) => {
                    let name = name.to_str().unwrap_or("anon").to_string();
                    if name.len() == 0 {
                        "anon".to_string()
                    } else {
                        name
                    }
                }
                Err(e) => {
                    eprintln!("websocket error: {}", e);
                    return
                }
            };
        } else {
            return
        }

        let user = User::new(&username);

        let msg: String;
        {
            let mut connected = self.connected.lock().await;
            *connected += 1;
            msg = format!("{} just joined the room! There are now {} user(s) connected.", user.name, *connected);
        }

        {
            // register this user in the server state
            self.users.lock().await.insert(user.name.clone(), tx);
        }

        self.broadcast(&msg, &self.admin).await;


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
                    eprintln!("websocket error(id={}): {}", user.name, e);
                    break;
                }
            };
            let str: String = if msg.is_text() {
                msg.to_str().unwrap().to_string()
            } else {
                "error".to_string()
            };
            self.broadcast(&str, &user).await;
        }

        // the while loop will continue as long as the websocket is open
        // when it closes, we finally reach this disconnect function call
        self.disconnect_user(user.name).await;
    }
}
