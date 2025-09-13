use crate::auth::{JwtAuthProvider, WebSocketAuth, TopicAccessControl};
use crate::broadcast::{BroadcastSystem, BroadcastConfig};
use crate::channel::ChannelManager;
use crate::client::{WebSocketClient, ClientConnection, ClientStats};
use crate::events::WebSocketMessage;
use crate::handler::{MessageHandler, DefaultGameHandler, GameMessageHandler};
use dashmap::DashMap;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Client connection timeout
    pub connection_timeout: Duration,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// Broadcast system configuration
    pub broadcast_config: BroadcastConfig,
    /// Enable connection statistics
    pub enable_stats: bool,
    /// Stats collection interval
    pub stats_interval: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:4000".parse().unwrap(),
            max_connections: 10000,
            connection_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
            jwt_secret: "change-this-in-production".to_string(),
            broadcast_config: BroadcastConfig::default(),
            enable_stats: true,
            stats_interval: Duration::from_secs(60),
        }
    }
}

/// Main WebSocket server
pub struct WebSocketServer {
    config: ServerConfig,
    clients: Arc<DashMap<Uuid, Arc<WebSocketClient>>>,
    channel_manager: Arc<ChannelManager>,
    broadcast_system: Arc<BroadcastSystem>,
    message_handler: Arc<MessageHandler>,
    server_stats: Arc<RwLock<ServerStats>>,
    shutdown_sender: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone, Default)]
pub struct ServerStats {
    pub total_connections: u64,
    pub active_connections: usize,
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub uptime: Duration,
    pub start_time: std::time::Instant,
    pub peak_connections: usize,
    pub total_channels: usize,
    pub total_broadcasts: u64,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: ServerConfig) -> Self {
        Self::new_with_game_handler(config, None)
    }

    /// Create a new WebSocket server with custom game handler
    pub fn new_with_game_handler(
        config: ServerConfig,
        game_handler: Option<Arc<dyn GameMessageHandler>>,
    ) -> Self {
        // Set up authentication
        let auth_provider = Arc::new(JwtAuthProvider::new(config.jwt_secret.clone()));
        let auth = Arc::new(WebSocketAuth::new(auth_provider));
        let access_control = Arc::new(TopicAccessControl::new(auth.clone()));

        // Set up channel management
        let channel_manager = Arc::new(ChannelManager::new(access_control));

        // Set up broadcast system
        let broadcast_system = Arc::new(BroadcastSystem::new(config.broadcast_config.clone()));

        // Set up game handler
        let game_handler = game_handler.unwrap_or_else(|| {
            Arc::new(DefaultGameHandler::new(broadcast_system.clone()))
        });

        // Set up message handler
        let message_handler = Arc::new(MessageHandler::new(
            auth,
            channel_manager.clone(),
            broadcast_system.clone(),
            game_handler,
        ));

        Self {
            config,
            clients: Arc::new(DashMap::new()),
            channel_manager,
            broadcast_system,
            message_handler,
            server_stats: Arc::new(RwLock::new(ServerStats {
                start_time: std::time::Instant::now(),
                ..Default::default()
            })),
            shutdown_sender: None,
        }
    }

    /// Start the WebSocket server
    pub async fn start(&mut self) -> Result<(), ServerError> {
        info!("Starting WebSocket server on {}", self.config.bind_address);

        // Start broadcast system
        let _broadcast_receiver = self.broadcast_system.start().await;

        // Set up shutdown channel
        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);
        self.shutdown_sender = Some(shutdown_sender);

        // Bind to address
        let listener = TcpListener::bind(self.config.bind_address)
            .await
            .map_err(|e| ServerError::BindError(e))?;

        info!("WebSocket server listening on {}", self.config.bind_address);

        // Start background tasks
        self.start_background_tasks().await;

        // Main accept loop
        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            if let Err(e) = self.handle_new_connection(stream, addr).await {
                                error!("Error handling new connection from {}: {}", addr, e);
                            }
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }

                // Handle shutdown signal
                _ = shutdown_receiver.recv() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        self.shutdown().await;
        Ok(())
    }

    /// Shutdown the server gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down WebSocket server");

        // Disconnect all clients
        for client_entry in self.clients.iter() {
            let client = client_entry.value();
            if let Err(e) = client.send_message(WebSocketMessage::new(
                "system".to_string(),
                "server_shutdown".to_string(),
                serde_json::json!({ "message": "Server is shutting down" }),
            )).await {
                warn!("Failed to send shutdown message to client {}: {}", client.id, e);
            }
        }

        // Clean up all clients
        self.clients.clear();

        info!("WebSocket server shutdown complete");
    }

    /// Send shutdown signal to the server
    pub async fn send_shutdown_signal(&self) -> Result<(), ServerError> {
        if let Some(sender) = &self.shutdown_sender {
            sender.send(()).await.map_err(|_| ServerError::ShutdownError)?;
        }
        Ok(())
    }

    /// Handle new WebSocket connection
    async fn handle_new_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), ServerError> {
        // Check connection limits
        if self.clients.len() >= self.config.max_connections {
            warn!("Connection limit reached, rejecting connection from {}", addr);
            return Err(ServerError::ConnectionLimitReached);
        }

        // Accept WebSocket connection
        let websocket = accept_async(stream)
            .await
            .map_err(ServerError::WebSocketError)?;

        // Create client
        let client_id = Uuid::new_v4();
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut client = WebSocketClient::new(client_id, sender);

        // Set client metadata
        {
            let mut metadata = client.metadata.write().await;
            metadata.ip_address = addr.ip().to_string();
        }

        let client = Arc::new(client);

        // Register client
        self.clients.insert(client_id, client.clone());

        // Update stats
        {
            let mut stats = self.server_stats.write().await;
            stats.total_connections += 1;
            stats.active_connections = self.clients.len();
            if stats.active_connections > stats.peak_connections {
                stats.peak_connections = stats.active_connections;
            }
        }

        info!("New WebSocket connection from {}: {}", addr, client_id);

        // Create connection handler
        let connection = ClientConnection::new(client.clone(), websocket, receiver);

        // Start connection handler in background
        let clients = self.clients.clone();
        let message_handler = self.message_handler.clone();
        let server_stats = self.server_stats.clone();

        tokio::spawn(async move {
            // Handle connection
            let result = connection.run().await;
            
            if let Err(e) = result {
                warn!("Connection {} ended with error: {}", client_id, e);
            }

            // Clean up client
            clients.remove(&client_id);

            // Update stats
            {
                let mut stats = server_stats.write().await;
                stats.active_connections = clients.len();
            }

            info!("Client {} disconnected", client_id);
        });

        // Handle client messages
        let client_for_messages = client.clone();
        let message_handler_for_messages = message_handler.clone();
        let server_stats_for_messages = server_stats.clone();

        tokio::spawn(async move {
            let mut message_stream = client_for_messages.sender.subscribe();
            
            while let Ok(message) = message_stream.recv().await {
                // Handle the message
                match message_handler_for_messages.handle_message(client_for_messages.clone(), message).await {
                    Ok(response) => {
                        if let Some(response_msg) = response {
                            if let Err(e) = client_for_messages.send_message(response_msg).await {
                                error!("Failed to send response to client {}: {}", client_for_messages.id, e);
                            }
                        }

                        // Update stats
                        {
                            let mut stats = server_stats_for_messages.write().await;
                            stats.messages_processed += 1;
                        }
                    }
                    Err(e) => {
                        error!("Error handling message from client {}: {}", client_for_messages.id, e);
                        
                        // Update stats
                        {
                            let mut stats = server_stats_for_messages.write().await;
                            stats.messages_failed += 1;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start background maintenance tasks
    async fn start_background_tasks(&self) {
        // Start stats collection task if enabled
        if self.config.enable_stats {
            self.start_stats_collection_task().await;
        }

        // Start client cleanup task
        self.start_client_cleanup_task().await;

        // Start heartbeat monitoring task
        self.start_heartbeat_monitoring_task().await;
    }

    /// Start statistics collection task
    async fn start_stats_collection_task(&self) {
        let server_stats = self.server_stats.clone();
        let clients = self.clients.clone();
        let channel_manager = self.channel_manager.clone();
        let broadcast_system = self.broadcast_system.clone();
        let interval_duration = self.config.stats_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                let mut stats = server_stats.write().await;
                stats.uptime = stats.start_time.elapsed();
                stats.active_connections = clients.len();
                stats.total_channels = channel_manager.get_active_channels().len();
                
                let broadcast_stats = broadcast_system.get_stats().await;
                stats.total_broadcasts = broadcast_stats.total_events;

                debug!("Server stats - Active connections: {}, Total messages: {}, Uptime: {:?}", 
                    stats.active_connections, stats.messages_processed, stats.uptime);
            }
        });
    }

    /// Start client cleanup task
    async fn start_client_cleanup_task(&self) {
        let clients = self.clients.clone();
        let channel_manager = self.channel_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Clean up disconnected clients
                let mut to_remove = Vec::new();
                
                for client_entry in clients.iter() {
                    let client = client_entry.value();
                    if !client.is_alive(Duration::from_secs(120)).await {
                        to_remove.push(client.id);
                    }
                }

                for client_id in to_remove {
                    if clients.remove(&client_id).is_some() {
                        channel_manager.remove_client_from_all_channels(client_id).await;
                        info!("Cleaned up inactive client: {}", client_id);
                    }
                }
            }
        });
    }

    /// Start heartbeat monitoring task
    async fn start_heartbeat_monitoring_task(&self) {
        let clients = self.clients.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(heartbeat_interval);

            loop {
                interval.tick().await;

                // Send heartbeat requests to all clients
                for client_entry in clients.iter() {
                    let client = client_entry.value();
                    
                    let heartbeat_msg = WebSocketMessage::new(
                        "phoenix".to_string(),
                        "phx_heartbeat".to_string(),
                        serde_json::json!({}),
                    );

                    if let Err(e) = client.send_message(heartbeat_msg).await {
                        warn!("Failed to send heartbeat to client {}: {}", client.id, e);
                    }
                }
            }
        });
    }

    /// Get current server statistics
    pub async fn get_stats(&self) -> ServerStats {
        let mut stats = self.server_stats.read().await.clone();
        stats.active_connections = self.clients.len();
        stats.total_channels = self.channel_manager.get_active_channels().len();
        
        let broadcast_stats = self.broadcast_system.get_stats().await;
        stats.total_broadcasts = broadcast_stats.total_events;
        
        stats
    }

    /// Get statistics for all connected clients
    pub async fn get_client_stats(&self) -> Vec<ClientStats> {
        let mut client_stats = Vec::new();
        
        for client_entry in self.clients.iter() {
            let client = client_entry.value();
            client_stats.push(client.get_stats().await);
        }
        
        client_stats
    }

    /// Get information about active channels
    pub fn get_channel_info(&self) -> Vec<String> {
        self.channel_manager.get_active_channels()
    }

    /// Broadcast an event to all connected clients
    pub async fn broadcast_global_event(
        &self,
        event: crate::events::GameEvent,
    ) -> Result<(), ServerError> {
        self.broadcast_system
            .broadcast_event(event)
            .await
            .map_err(ServerError::BroadcastError)?;
        Ok(())
    }

    /// Get a specific client by ID
    pub fn get_client(&self, client_id: Uuid) -> Option<Arc<WebSocketClient>> {
        self.clients.get(&client_id).map(|entry| entry.value().clone())
    }

    /// Get number of active connections
    pub fn connection_count(&self) -> usize {
        self.clients.len()
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    BindError(std::io::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Connection limit reached")]
    ConnectionLimitReached,
    
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] crate::broadcast::BroadcastError),
    
    #[error("Shutdown error")]
    ShutdownError,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig {
            bind_address: "127.0.0.1:0".parse().unwrap(), // Use any available port
            ..Default::default()
        };
        
        let server = WebSocketServer::new(config);
        assert_eq!(server.connection_count(), 0);
        
        let stats = server.get_stats().await;
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_connections, 0);
    }

    #[test]
    fn test_server_config_creation() {
        let config = ServerConfig::default();
        assert_eq!(config.max_connections, 10000);
        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert!(config.enable_stats);
    }
}