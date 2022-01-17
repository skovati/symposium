// 3rd party imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::{Filter, reject};
use warp::{reply, Reply};
use warp::{Rejection, http::StatusCode};
use sha2::{Sha256, Digest};
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
    ids: Arc<Mutex<HashMap<String, String>>>,
    addr: SocketAddr,
    connected: Arc<Mutex<usize>>,
    admin: User,
}

#[derive(Debug)]
struct InvalidUsername;

impl reject::Reject for InvalidUsername {}

impl Router {
    pub fn new(addr: SocketAddr) -> Self {
        Router {
            users: Arc::new(Mutex::new(HashMap::new())),
            ids: Arc::new(Mutex::new(HashMap::new())),
            addr,
            connected: Arc::new(Mutex::new(0)),
            admin: User {
                name: "server".to_string(),
                id: "".to_string(),
            }
        }
    }

    pub async fn run(&self) {
        let router = self.clone();
        let router = warp::any().map(move || router.clone());


        // GET / -> serves login.html
        let root = warp::path::end()
            .and(warp::get())
            .and(warp::fs::file("static/login.html"));

        // GET / -> serves login.html
        let chat = warp::path("chat")
            .and(warp::get())
            .and(warp::fs::file("static/chat.html"));

        // GET / -> serves static CSS and JS
        let files = warp::get()
            .and(warp::fs::dir("static"));

        let register = warp::path("register")
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json::<User>())
            .and(router.clone())
            .and_then(|user: User, router: Router| async move {
                // let users = router.users.lock().await;
                let mut ids = router.ids.lock().await;
                if !ids.contains_key(&user.name) {
                    let hash = format!("{:x}", Sha256::new()
                        .chain_update(user.name.clone())
                        .finalize());
                    ids.insert(user.name.clone(), hash.clone());
                    Ok(warp::reply::json( &User {
                        name: user.name.clone(),
                        id: hash,
                    }))
                } else {
                    Err(warp::reject::custom(InvalidUsername))
                }
            });

        // GET /ws -> websocket upgrade
        let ws = warp::path("ws")
            // The `ws()` filter will prepare Websocket handshake...
            .and(warp::ws())
            .and(router)
            .map(|ws: warp::ws::Ws, router: Router| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| router.handle_user(socket))
            });

        let routes = root   // serve login.html
            .or(chat)       // or chat.html
            .or(files)      // or server static css and js
            .or(register)   // or respond to register api
            .or(ws);        // or open authenticated websocket connection

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

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if let Some(e) = err.find::<InvalidUsername>() {
        Ok(reply::with_status("CONFLICT", StatusCode::CONFLICT))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR))
    }
}
