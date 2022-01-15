// use serde::{Serialize, Deserialize};
use warp::ws::Message;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: &String) -> Self {
        User {
            name: name.clone(),
        }
    }
}
