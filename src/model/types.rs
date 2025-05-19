use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct ShortenResponse {
    pub short_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UrlEntry {
    pub original: String,
}

pub type UrlMap = Arc<Mutex<HashMap<String, UrlEntry>>>;
pub struct AppState {
    pub url_map: UrlMap,
}
