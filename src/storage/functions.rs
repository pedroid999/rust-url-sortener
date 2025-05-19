use crate::model::types::UrlEntry;
use std::collections::HashMap;
use std::io;

pub async fn load_db(path: &str) -> Result<HashMap<String, UrlEntry>, io::Error> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(data) => data,
        Err(_) => return Ok(HashMap::new()),
    };
    
    match serde_json::from_str(&content) {
        Ok(map) => Ok(map),
        Err(_) => Ok(HashMap::new()),
    }
}

pub async fn save_db(path: &str, map: &HashMap<String, UrlEntry>) -> Result<(), io::Error> {
    let json = serde_json::to_string_pretty(map).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    tokio::fs::write(path, json).await
}
