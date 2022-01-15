use chrono::Local;
use serde::{Serialize, Deserialize};
use crate::user::User;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parcel {
    payload: String,
    from: User,
    postmark: String,
}

impl Parcel {
    pub fn new(payload: &String, from: &User) -> Self {
        Parcel {
            payload: payload.clone(),
            from: from.clone(),
            postmark: Local::now().format("%I:%M").to_string(),
        }
    }
}
