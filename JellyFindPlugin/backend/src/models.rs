use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Torrent {
    pub title: String,
    pub url: String,
    pub seeders: Option<u32>,
    pub peers: Option<u32>,
    pub size: Option<f32>,
}