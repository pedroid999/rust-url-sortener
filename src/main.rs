use std::sync::{Arc, Mutex};
use actix_web::{App, HttpServer, web, middleware::Logger};
use std::collections::HashMap;
use std::io;

mod model;
mod routes;
mod storage;

use crate::model::types::AppState;
use crate::routes::functions::{shorten_url, redirect_url, dashboard};
use crate::storage::functions::load_db;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Ensure the data directory exists
    std::fs::create_dir_all("data").unwrap_or_else(|_| println!("Data directory already exists"));
    
    // Initialize the URL database
    let db_map = match load_db("data/db.json").await {
        Ok(map) => {
            println!("🔄 Database loaded successfully with {} entries", map.len());
            // Log the content of the map
            for (code, entry) in &map {
                println!("🔄 Loaded URL: code={}, original={}", code, entry.original);
            }
            map
        },
        Err(e) => {
            println!("❌ Error loading database: {}", e);
            println!("🔄 Starting with empty database");
            HashMap::new()
        }
    };
    
    // Create application state
    let app_state = web::Data::new(AppState {
        url_map: Arc::new(Mutex::new(db_map)),
    });
    
    println!("🚀 Starting server at http://localhost:8081");
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(dashboard)  // Primera para que no se confunda con un código corto
            .service(shorten_url)
            .service(redirect_url)  // Último para que procese los códigos cortos
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
