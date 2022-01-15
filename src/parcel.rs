use chrono::Local;
use serde::{Serialize, Deserialize};
use crate::user::User;

#[derive(Serialize, Deserialize, Debug)]
pub struct Parcel {
    payload: String,
    from: User,
    postmark: String,
}

impl Parcel {
    pub fn new(payload: String, from: User) -> Self {
        Parcel {
            payload,
            from,
            postmark: Local::now().format("%I:%M").to_string(),
        }
    }
}
