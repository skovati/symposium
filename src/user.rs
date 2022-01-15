// use serde::{Serialize, Deserialize};
use warp::ws::Message;
use tokio::sync::mpsc;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub tx: Tx,
}

impl User {
    pub fn new(name: &String, tx: Tx) -> Self {
        User {
            name: name.clone(),
            tx,
        }
    }
}
