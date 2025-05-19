use yew::prelude::*;
use gloo_net::http::Request;
use web_sys::HtmlInputElement;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
struct ShortenRequest {
    url: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ShortenResponse {
    short_url: String,
}

type DashboardData = HashMap<String, (String, u64)>;

#[function_component(App)]
fn app() -> Html {
    let url_input_ref = use_node_ref();
    let shortened_url = use_state(|| None::<String>);
    let dashboard_data = use_state(|| None::<DashboardData>);
    let error = use_state(|| None::<String>);
    let is_loading = use_state(|| true);
    
    let fetch_dashboard = {
        let dashboard_data = dashboard_data.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();
        
        Callback::from(move |_: MouseEvent| {
            let dashboard_data = dashboard_data.clone();
            let error = error.clone();
            let is_loading = is_loading.clone();
            
            is_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match Request::get("/dashboard").send().await {
                    Ok(response) => {
                        if response.status() == 200 {
                            match response.json::<DashboardData>().await {
                                Ok(data) => {
                                    dashboard_data.set(Some(data));
                                    error.set(None);
                                },
                                Err(e) => error.set(Some(format!("Failed to parse dashboard data: {}", e))),
                            }
                        } else {
                            // En caso de 404, simplemente no hay datos todavía
                            if response.status() == 404 {
                                dashboard_data.set(Some(HashMap::new()));
                                error.set(None);
                            } else {
                                error.set(Some(format!("Server error: {}", response.status())));
                            }
                        }
                    },
                    Err(e) => error.set(Some(format!("Network error: {}", e))),
                }
                is_loading.set(false);
            });
        })
    };
    
    // Fetch dashboard data on component mount
    {
        let dashboard_data = dashboard_data.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();
        
        use_effect_with_deps(
            move |_| {
                let dashboard_data = dashboard_data.clone();
                let error = error.clone();
                let is_loading = is_loading.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("Fetching dashboard data...");
                    match Request::get("/dashboard").send().await {
                        Ok(response) => {
                            log::info!("Dashboard response status: {}", response.status());
                            if response.status() == 200 {
                                match response.text().await {
                                    Ok(text) => {
                                        log::info!("Dashboard response text: {}", text);
                                        match serde_json::from_str::<DashboardData>(&text) {
                                            Ok(data) => {
                                                log::info!("Dashboard data parsed successfully: {:?}", data);
                                                dashboard_data.set(Some(data));
                                                error.set(None);
                                            },
                                            Err(e) => {
                                                log::error!("Failed to parse dashboard data: {}", e);
                                                error.set(Some(format!("Failed to parse dashboard data: {}. Raw text: {}", e, text)));
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        log::error!("Failed to get response text: {}", e);
                                        error.set(Some(format!("Failed to get response text: {}", e)));
                                    }
                                }
                            } else {
                                // En caso de 404, simplemente no hay datos todavía
                                if response.status() == 404 {
                                    log::warn!("Dashboard returned 404");
                                    dashboard_data.set(Some(HashMap::new()));
                                    error.set(None);
                                } else {
                                    log::error!("Server error: {}", response.status());
                                    error.set(Some(format!("Server error: {}", response.status())));
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("Network error: {}", e);
                            error.set(Some(format!("Network error: {}", e)));
                        }
                    }
                    is_loading.set(false);
                });
                || ()
            },
            (),
        );
    }
    
    // Función para actualizar el dashboard
    let update_dashboard = {
        let dashboard_data = dashboard_data.clone();
        
        Callback::from(move |_| {
            let dashboard_data = dashboard_data.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/dashboard").send().await {
                    if response.status() == 200 {
                        if let Ok(data) = response.json::<DashboardData>().await {
                            dashboard_data.set(Some(data));
                        }
                    }
                }
            });
        })
    };
    
    let on_submit = {
        let url_input_ref = url_input_ref.clone();
        let shortened_url = shortened_url.clone();
        let error = error.clone();
        let update_dashboard = update_dashboard.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let input = url_input_ref.cast::<HtmlInputElement>().unwrap();
            let url_value = input.value();
            
            if url_value.trim().is_empty() {
                error.set(Some("Please enter a URL".to_string()));
                return;
            }
            
            if !url_value.starts_with("http://") && !url_value.starts_with("https://") {
                error.set(Some("URL must start with http:// or https://".to_string()));
                return;
            }
            
            let shortened_url = shortened_url.clone();
            let error = error.clone();
            let update_dashboard = update_dashboard.clone();
            let request = ShortenRequest { url: url_value };
            
            wasm_bindgen_futures::spawn_local(async move {
                match Request::post("/shorten")
                    .json(&request)
                    .expect("Failed to serialize request")
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status() == 200 {
                            match response.json::<ShortenResponse>().await {
                                Ok(data) => {
                                    // Guardamos la URL original para mostrarla en el frontend
                                    shortened_url.set(Some(data.short_url));
                                    error.set(None);
                                    
                                    // Actualizar dashboard después de acortar una URL
                                    update_dashboard.emit(());
                                },
                                Err(e) => error.set(Some(format!("Failed to parse response: {}", e))),
                            }
                        } else {
                            error.set(Some(format!("Server error: {}", response.status())));
                        }
                    },
                    Err(e) => error.set(Some(format!("Network error: {}", e))),
                }
            });
            
            input.set_value("");
        })
    };
    
    let redirect_to_backend = {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            // Obtener href del enlace
            if let Some(a_element) = e.target_dyn_into::<web_sys::HtmlAnchorElement>() {
                let href = a_element.href();
                // Extraer el código de la URL
                if let Some(code) = href.split('/').last() {
                    // Redirigir al backend directamente
                    let backend_url = format!("http://localhost:8081/{}", code);
                    let _ = web_sys::window().unwrap().location().set_href(&backend_url);
                }
            }
        })
    };
    
    html! {
        <div>
            <h1>{"URL Shortener"}</h1>
            
            <div class="form-container">
                <h2>{"Shorten a URL"}</h2>
                <form onsubmit={on_submit}>
                    <input 
                        type="text"
                        ref={url_input_ref}
                        placeholder="Enter a URL (https://example.com)" 
                    />
                    <button type="submit">{"Shorten"}</button>
                </form>
                
                if let Some(url) = (*shortened_url).clone() {
                    <div class="result">
                        <p>{"Your shortened URL:"}</p>
                        <a href={url.clone()} onclick={redirect_to_backend.clone()} target="_blank">{url}</a>
                    </div>
                }
                
                if let Some(err) = (*error).clone() {
                    <div class="error">
                        <p>{err}</p>
                    </div>
                }
            </div>
            
            <div class="dashboard">
                <h2>{"Dashboard"}</h2>
                <button onclick={fetch_dashboard}>{"Refresh Data"}</button>
                
                if let Some(data) = (*dashboard_data).clone() {
                    if data.is_empty() {
                        <p>{"No shortened URLs yet."}</p>
                    } else {
                        <table>
                            <thead>
                                <tr>
                                    <th>{"Short Code"}</th>
                                    <th>{"Original URL"}</th>
                                    <th>{"Clicks"}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {for data.iter().map(|(code, (original_url, clicks))| {
                                    let backend_url = format!("http://localhost:8081/{}", code);
                                    html! {
                                        <tr class="url-card">
                                            <td><a href={backend_url.clone()} onclick={redirect_to_backend.clone()} target="_blank">{code}</a></td>
                                            <td><a href={original_url.clone()} target="_blank">{original_url.clone()}</a></td>
                                            <td>{clicks}</td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    }
                } else if *is_loading {
                    <p>{"Loading dashboard data..."}</p>
                }
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
