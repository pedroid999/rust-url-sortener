use crate::model::types::{ShortenResponse, ShortenRequest, UrlEntry, AppState};
use crate::storage::functions::save_db;
use actix_web::{post, get, web, HttpResponse, Responder};
use std::collections::HashMap;
use uuid::Uuid;

#[post("/shorten")]
async fn shorten_url(
    body: web::Json<ShortenRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let url_to_shorten = body.url.clone();
    
    // Verificar si la URL ya existe
    let mut url_map = data.url_map.lock().unwrap();
    
    // Buscar si la URL original ya existe en el sistema
    for (existing_code, entry) in url_map.iter() {
        if entry.original == url_to_shorten {
            // La URL ya existe, devolvemos el código existente
            return HttpResponse::Ok().json(ShortenResponse {
                short_url: format!("http://localhost:8081/{}", existing_code),
            });
        }
    }
    
    // La URL no existe, generamos un nuevo código
    let code = Uuid::new_v4().to_string()[..8].to_string();
    let entry = UrlEntry {
        original: url_to_shorten,
    };
    
    url_map.insert(code.clone(), entry);
    
    if let Err(e) = save_db("data/db.json", &url_map).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    
    // Initialize click count
    let mut clicks: HashMap<String, u64> = tokio::fs::read_to_string("data/clicks.json")
        .await
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    clicks.insert(code.clone(), 0);
    let _ = tokio::fs::write("data/clicks.json", serde_json::to_string_pretty(&clicks).unwrap()).await;
    
    HttpResponse::Ok().json(ShortenResponse {
        short_url: format!("http://localhost:8081/{}", code),
    })
}

// La ruta de redirección debe ir después de todas las demás rutas API
#[get("/{code}")]
async fn redirect_url(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let code = path.into_inner();
    
    // Verificar si es una ruta de API
    if code == "dashboard" || code == "shorten" {
        return HttpResponse::NotFound().body("Route not found");
    }
    
    let map = data.url_map.lock().unwrap();
    if let Some(entry) = map.get(&code) {
        let mut clicks: HashMap<String, u64> = tokio::fs::read_to_string("data/clicks.json")
            .await
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let count = clicks.entry(code.clone()).or_insert(0);
        *count += 1;
        let _ = tokio::fs::write("data/clicks.json", serde_json::to_string_pretty(&clicks).unwrap()).await;
        HttpResponse::Found().append_header(("Location", entry.original.clone())).finish()
    } else {
        HttpResponse::NotFound().body("URL not found")
    }
}

// Define el endpoint dashboard con una ruta explícita
#[get("/dashboard")]
async fn dashboard(data: web::Data<AppState>) -> impl Responder {
    println!("⭐ Dashboard endpoint called");
    
    let map = data.url_map.lock().unwrap();
    let clicks: HashMap<String, u64> = tokio::fs::read_to_string("data/clicks.json")
        .await
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    let mut report = HashMap::new();
    
    for (code, entry) in map.iter() {
        let click_count = clicks.get(code).cloned().unwrap_or(0);
        report.insert(code.clone(), (entry.original.clone(), click_count));
    }
    
    println!("⭐ Dashboard report: {:?}", report);
    
    HttpResponse::Ok().json(report)
}
