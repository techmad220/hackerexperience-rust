//! HackerExperience Game Server - Main Entry Point

mod game_server;
mod websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Start the server
    game_server::run_server().await
}