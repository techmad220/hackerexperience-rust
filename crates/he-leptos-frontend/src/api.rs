//! API client for backend communication

use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

const API_BASE: &str = "http://localhost:3005/api";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// Generic API request function
pub async fn api_request<T: for<'a> Deserialize<'a>>(
    method: &str,
    endpoint: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, endpoint);

    let mut request = gloo_net::http::Request::new(&url)
        .method(match method {
            "GET" => gloo_net::http::Method::GET,
            "POST" => gloo_net::http::Method::POST,
            "PUT" => gloo_net::http::Method::PUT,
            "DELETE" => gloo_net::http::Method::DELETE,
            _ => gloo_net::http::Method::GET,
        })
        .header("Content-Type", "application/json");

    // Add auth token if available
    if let Ok(Some(token)) = window().local_storage() {
        if let Ok(Some(token)) = token.get_item("auth_token") {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }
    }

    // Add body if provided
    if let Some(body_data) = body {
        request = request.body(body_data.to_string());
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if response.ok() {
        response.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("API request failed: {}", response.status()))
    }
}

// Authentication APIs
pub async fn login(email: &str, password: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct LoginResponse {
        success: bool,
        token: Option<String>,
        message: String,
    }

    let response = api_request::<LoginResponse>(
        "POST",
        "/auth/login",
        Some(json!({
            "email": email,
            "password": password
        }))
    ).await?;

    if response.success {
        if let Some(token) = response.token {
            // Store token in localStorage
            if let Ok(Some(storage)) = window().local_storage() {
                let _ = storage.set_item("auth_token", &token);
            }
            Ok(token)
        } else {
            Err("No token received".to_string())
        }
    } else {
        Err(response.message)
    }
}

pub async fn register(username: &str, email: &str, password: &str) -> Result<(), String> {
    #[derive(Deserialize)]
    struct RegisterResponse {
        success: bool,
        message: String,
    }

    let response = api_request::<RegisterResponse>(
        "POST",
        "/auth/register",
        Some(json!({
            "login": username,
            "email": email,
            "password": password
        }))
    ).await?;

    if response.success {
        Ok(())
    } else {
        Err(response.message)
    }
}

// Process APIs
pub async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    #[derive(Deserialize)]
    struct ProcessResponse {
        success: bool,
        processes: Vec<ProcessInfo>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProcessInfo {
        pub pid: i64,
        pub process_type: String,
        pub pc_id: String,
        pub target_pc_id: Option<String>,
        pub start_time: String,
        pub end_time: String,
        pub priority: i32,
    }

    let response = api_request::<ProcessResponse>("GET", "/processes", None).await?;

    if response.success {
        Ok(response.processes)
    } else {
        Err("Failed to get processes".to_string())
    }
}

pub async fn create_process(process_type: &str, target_pc: Option<String>) -> Result<i64, String> {
    #[derive(Deserialize)]
    struct CreateResponse {
        success: bool,
        pid: i64,
        message: String,
    }

    let response = api_request::<CreateResponse>(
        "POST",
        "/processes",
        Some(json!({
            "process_type": process_type,
            "target_pc_id": target_pc
        }))
    ).await?;

    if response.success {
        Ok(response.pid)
    } else {
        Err(response.message)
    }
}

pub async fn cancel_process(pid: i64) -> Result<(), String> {
    #[derive(Deserialize)]
    struct CancelResponse {
        success: bool,
        message: String,
    }

    let response = api_request::<CancelResponse>(
        "DELETE",
        &format!("/processes/{}/cancel", pid),
        None
    ).await?;

    if response.success {
        Ok(())
    } else {
        Err(response.message)
    }
}

// Hardware APIs
pub async fn get_hardware() -> Result<HardwareInfo, String> {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HardwareInfo {
        pub cpu_speed: f64,
        pub ram_size: i64,
        pub hdd_size: i64,
        pub hdd_used: i64,
        pub net_speed: f64,
    }

    api_request::<HardwareInfo>("GET", "/hardware", None).await
}

pub async fn upgrade_hardware(component: &str, level: u32) -> Result<(), String> {
    #[derive(Deserialize)]
    struct UpgradeResponse {
        success: bool,
        message: String,
    }

    let response = api_request::<UpgradeResponse>(
        "POST",
        "/hardware/upgrade",
        Some(json!({
            "component": component,
            "level": level
        }))
    ).await?;

    if response.success {
        Ok(())
    } else {
        Err(response.message)
    }
}

// Dashboard API
pub async fn get_dashboard() -> Result<DashboardData, String> {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DashboardData {
        pub status: GameStatus,
        pub active_processes: usize,
        pub hardware_load: f32,
        pub bank_balance: i64,
        pub unread_messages: u32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct GameStatus {
        pub online: bool,
        pub level: u32,
        pub experience: u64,
        pub reputation: i32,
    }

    api_request::<DashboardData>("GET", "/game/dashboard", None).await
}

// WebSocket connection
pub fn connect_websocket() -> Result<web_sys::WebSocket, String> {
    use wasm_bindgen::prelude::*;
    use web_sys::{WebSocket, MessageEvent};

    let ws = WebSocket::new("ws://localhost:3005/ws").map_err(|e| format!("{:?}", e))?;

    // Set binary type
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // Setup event handlers
    let onopen = Closure::<dyn FnMut()>::new(move || {
        web_sys::console::log_1(&"WebSocket connected".into());
    });

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            web_sys::console::log_1(&format!("WS message: {}", txt.as_string().unwrap()).into());
        }
    });

    let onerror = Closure::<dyn FnMut()>::new(move || {
        web_sys::console::error_1(&"WebSocket error".into());
    });

    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    // Keep closures alive
    onopen.forget();
    onmessage.forget();
    onerror.forget();

    Ok(ws)
}