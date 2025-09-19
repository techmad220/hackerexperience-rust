//! HackerExperience API Server

pub mod middleware;
pub mod handlers;
pub mod routes;
pub mod config;
pub mod openapi;

// Re-export main types
pub use config::ApiConfig;

pub fn hello() {
    println!("Hello from he-api!");
}