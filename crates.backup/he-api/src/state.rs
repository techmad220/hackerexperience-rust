//! Application state

use he_database::Database;
use he_auth::AuthService;
use he_game_world::GameWorld;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth: AuthService,
    pub ws_manager: Option<Arc<he_websocket::ConnectionManager>>,
    pub game_world: Arc<RwLock<GameWorld>>,
}