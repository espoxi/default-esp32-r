use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creds {
    pub ssid: String,
    pub psk: String,
}