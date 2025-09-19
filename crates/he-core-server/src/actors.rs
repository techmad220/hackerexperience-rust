//! Server Actor System
//!
//! This module provides actor implementations for server management,
//! including server lifecycle, component management, and resource monitoring.

use crate::{Server, ServerType, Component, Motherboard, Resources, ServerId, MotherboardId, ComponentId, ComponentType, Hostname, Password};
use he_core_core::actors::{Actor, ActorContext, Handler, Message};
use he_core_core::{CoreError, ProcessId};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, error, warn, debug};
use serde::{Serialize, Deserialize};

/// Messages for Server Actor
#[derive(Debug)]
pub struct CreateServer {
    pub server_type: ServerType,
    pub hostname: Hostname,
    pub password: Password,
}

impl Message for CreateServer {
    type Result = Result<Server, CoreError>;
}

#[derive(Debug)]
pub struct GetServer {
    pub server_id: ServerId,
}

impl Message for GetServer {
    type Result = Result<Option<Server>, CoreError>;
}

#[derive(Debug)]
pub struct UpdateServer {
    pub server_id: ServerId,
    pub hostname: Option<Hostname>,
    pub password: Option<Password>,
}

impl Message for UpdateServer {
    type Result = Result<Server, CoreError>;
}

#[derive(Debug)]
pub struct DeleteServer {
    pub server_id: ServerId,
}

impl Message for DeleteServer {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct AttachComponent {
    pub server_id: ServerId,
    pub component: Component,
}

impl Message for AttachComponent {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct DetachComponent {
    pub server_id: ServerId,
    pub component_id: ComponentId,
}

impl Message for DetachComponent {
    type Result = Result<Component, CoreError>;
}

#[derive(Debug)]
pub struct GetServerComponents {
    pub server_id: ServerId,
}

impl Message for GetServerComponents {
    type Result = Result<Vec<Component>, CoreError>;
}

#[derive(Debug)]
pub struct GetServerResources {
    pub server_id: ServerId,
}

impl Message for GetServerResources {
    type Result = Result<Resources, CoreError>;
}

#[derive(Debug)]
pub struct StartServer {
    pub server_id: ServerId,
}

impl Message for StartServer {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct StopServer {
    pub server_id: ServerId,
}

impl Message for StopServer {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct RebootServer {
    pub server_id: ServerId,
}

impl Message for RebootServer {
    type Result = Result<(), CoreError>;
}

/// Server state for tracking server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerState {
    Offline,
    Starting,
    Online,
    Stopping,
    Rebooting,
    Maintenance,
    Error(String),
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState::Offline
    }
}

/// Server Actor - manages server lifecycle and operations
#[derive(Debug)]
pub struct ServerActor {
    /// In-memory server storage (in production, this would be database-backed)
    servers: Arc<RwLock<HashMap<ServerId, Server>>>,
    /// Server components mapping
    components: Arc<RwLock<HashMap<ServerId, Vec<Component>>>>,
    /// Server states
    states: Arc<RwLock<HashMap<ServerId, ServerState>>>,
    /// Server motherboards
    motherboards: Arc<RwLock<HashMap<ServerId, Motherboard>>>,
}

impl ServerActor {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            components: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            motherboards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new unique server ID
    fn generate_server_id(&self) -> ServerId {
        ServerId::new()
    }

    /// Calculate total server resources from components
    async fn calculate_server_resources(&self, server_id: &ServerId) -> Resources {
        let components = self.components.read().await;
        if let Some(server_components) = components.get(server_id) {
            server_components.iter().fold(Resources::default(), |acc, component| {
                acc + component.get_resources()
            })
        } else {
            Resources::default()
        }
    }

    /// Validate server can accept component
    async fn can_attach_component(&self, server_id: &ServerId, component: &Component) -> bool {
        let motherboards = self.motherboards.read().await;
        let components = self.components.read().await;
        
        // Check if server has a motherboard
        if let Some(motherboard) = motherboards.get(server_id) {
            if let Some(server_components) = components.get(server_id) {
                // Check component compatibility and slot availability
                motherboard.can_attach_component(component, server_components)
            } else {
                // No components attached yet, check basic compatibility
                motherboard.is_compatible_component(component)
            }
        } else {
            // Server has no motherboard, can only attach motherboard
            matches!(component.component_type, ComponentType::Motherboard)
        }
    }
}

impl Actor for ServerActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("ServerActor started with process_id: {}", ctx.process_id);
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("ServerActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: CoreError, ctx: &mut ActorContext) {
        error!("ServerActor error on process_id {}: {}", ctx.process_id, err);
    }
}

#[async_trait]
impl Handler<CreateServer> for ServerActor {
    async fn handle(&mut self, msg: CreateServer, _ctx: &mut ActorContext) -> Result<Server, CoreError> {
        info!("Creating server with hostname: {}", msg.hostname);
        
        let mut servers = self.servers.write().await;
        let mut states = self.states.write().await;
        let mut components = self.components.write().await;
        
        let server_id = self.generate_server_id();
        let server = Server::new(server_id, msg.server_type, msg.hostname, msg.password);
        
        // Initialize server state and components
        servers.insert(server_id, server.clone());
        states.insert(server_id, ServerState::Offline);
        components.insert(server_id, Vec::new());
        
        info!("Server created successfully: {}", server_id);
        Ok(server)
    }
}

#[async_trait]
impl Handler<GetServer> for ServerActor {
    async fn handle(&mut self, msg: GetServer, _ctx: &mut ActorContext) -> Result<Option<Server>, CoreError> {
        let servers = self.servers.read().await;
        Ok(servers.get(&msg.server_id).cloned())
    }
}

#[async_trait]
impl Handler<UpdateServer> for ServerActor {
    async fn handle(&mut self, msg: UpdateServer, _ctx: &mut ActorContext) -> Result<Server, CoreError> {
        let mut servers = self.servers.write().await;
        
        let server = servers.get_mut(&msg.server_id)
            .ok_or_else(|| CoreError::not_found("Server not found"))?;
        
        // Update fields if provided
        if let Some(hostname) = msg.hostname {
            server.hostname = hostname;
        }
        
        if let Some(password) = msg.password {
            server.password = password;
        }
        
        server.updated_at = Utc::now();
        
        info!("Server updated: {}", msg.server_id);
        Ok(server.clone())
    }
}

#[async_trait]
impl Handler<DeleteServer> for ServerActor {
    async fn handle(&mut self, msg: DeleteServer, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut servers = self.servers.write().await;
        let mut states = self.states.write().await;
        let mut components = self.components.write().await;
        let mut motherboards = self.motherboards.write().await;
        
        if servers.remove(&msg.server_id).is_some() {
            // Clean up all related data
            states.remove(&msg.server_id);
            components.remove(&msg.server_id);
            motherboards.remove(&msg.server_id);
            
            info!("Server deleted: {}", msg.server_id);
            Ok(())
        } else {
            Err(CoreError::not_found("Server not found"))
        }
    }
}

#[async_trait]
impl Handler<AttachComponent> for ServerActor {
    async fn handle(&mut self, msg: AttachComponent, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        debug!("Attaching component {:?} to server {}", msg.component.component_type, msg.server_id);
        
        // Check if component can be attached
        if !self.can_attach_component(&msg.server_id, &msg.component).await {
            return Err(CoreError::validation("Component cannot be attached to this server"));
        }
        
        let mut components = self.components.write().await;
        let mut servers = self.servers.write().await;
        let mut motherboards = self.motherboards.write().await;
        
        // Special handling for motherboard
        if matches!(msg.component.component_type, ComponentType::Motherboard) {
            if let Some(motherboard_data) = &msg.component.motherboard_data {
                motherboards.insert(msg.server_id, motherboard_data.clone());
                // Update server to reference motherboard
                if let Some(server) = servers.get_mut(&msg.server_id) {
                    server.attach_motherboard(msg.component.component_id);
                }
            }
        }
        
        // Add component to server
        let server_components = components.entry(msg.server_id).or_insert_with(Vec::new);
        server_components.push(msg.component);
        
        info!("Component attached to server: {}", msg.server_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<DetachComponent> for ServerActor {
    async fn handle(&mut self, msg: DetachComponent, _ctx: &mut ActorContext) -> Result<Component, CoreError> {
        let mut components = self.components.write().await;
        let mut servers = self.servers.write().await;
        let mut motherboards = self.motherboards.write().await;
        
        let server_components = components.get_mut(&msg.server_id)
            .ok_or_else(|| CoreError::not_found("Server not found"))?;
        
        // Find and remove component
        let component_index = server_components
            .iter()
            .position(|c| c.component_id == msg.component_id)
            .ok_or_else(|| CoreError::not_found("Component not found"))?;
        
        let component = server_components.remove(component_index);
        
        // Special handling for motherboard removal
        if matches!(component.component_type, ComponentType::Motherboard) {
            motherboards.remove(&msg.server_id);
            if let Some(server) = servers.get_mut(&msg.server_id) {
                server.detach_motherboard();
            }
        }
        
        info!("Component detached from server: {}", msg.server_id);
        Ok(component)
    }
}

#[async_trait]
impl Handler<GetServerComponents> for ServerActor {
    async fn handle(&mut self, msg: GetServerComponents, _ctx: &mut ActorContext) -> Result<Vec<Component>, CoreError> {
        let components = self.components.read().await;
        Ok(components.get(&msg.server_id).cloned().unwrap_or_default())
    }
}

#[async_trait]
impl Handler<GetServerResources> for ServerActor {
    async fn handle(&mut self, msg: GetServerResources, _ctx: &mut ActorContext) -> Result<Resources, CoreError> {
        Ok(self.calculate_server_resources(&msg.server_id).await)
    }
}

#[async_trait]
impl Handler<StartServer> for ServerActor {
    async fn handle(&mut self, msg: StartServer, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut states = self.states.write().await;
        let servers = self.servers.read().await;
        
        if !servers.contains_key(&msg.server_id) {
            return Err(CoreError::not_found("Server not found"));
        }
        
        let current_state = states.get(&msg.server_id).cloned().unwrap_or_default();
        
        match current_state {
            ServerState::Offline | ServerState::Error(_) => {
                states.insert(msg.server_id, ServerState::Starting);
                info!("Server {} starting", msg.server_id);
                
                // Simulate startup process
                tokio::spawn({
                    let server_id = msg.server_id;
                    let states = self.states.clone();
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        let mut states = states.write().await;
                        states.insert(server_id, ServerState::Online);
                        info!("Server {} is now online", server_id);
                    }
                });
                
                Ok(())
            }
            _ => Err(CoreError::validation("Server cannot be started in current state"))
        }
    }
}

#[async_trait]
impl Handler<StopServer> for ServerActor {
    async fn handle(&mut self, msg: StopServer, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut states = self.states.write().await;
        let servers = self.servers.read().await;
        
        if !servers.contains_key(&msg.server_id) {
            return Err(CoreError::not_found("Server not found"));
        }
        
        let current_state = states.get(&msg.server_id).cloned().unwrap_or_default();
        
        match current_state {
            ServerState::Online | ServerState::Starting => {
                states.insert(msg.server_id, ServerState::Stopping);
                info!("Server {} stopping", msg.server_id);
                
                // Simulate shutdown process
                tokio::spawn({
                    let server_id = msg.server_id;
                    let states = self.states.clone();
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        let mut states = states.write().await;
                        states.insert(server_id, ServerState::Offline);
                        info!("Server {} is now offline", server_id);
                    }
                });
                
                Ok(())
            }
            _ => Err(CoreError::validation("Server cannot be stopped in current state"))
        }
    }
}

#[async_trait]
impl Handler<RebootServer> for ServerActor {
    async fn handle(&mut self, msg: RebootServer, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut states = self.states.write().await;
        let servers = self.servers.read().await;
        
        if !servers.contains_key(&msg.server_id) {
            return Err(CoreError::not_found("Server not found"));
        }
        
        states.insert(msg.server_id, ServerState::Rebooting);
        info!("Server {} rebooting", msg.server_id);
        
        // Simulate reboot process
        tokio::spawn({
            let server_id = msg.server_id;
            let states = self.states.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                let mut states = states.write().await;
                states.insert(server_id, ServerState::Online);
                info!("Server {} reboot complete", server_id);
            }
        });
        
        Ok(())
    }
}

/// Server Supervisor - manages multiple server actors and provides supervision
#[derive(Debug)]
pub struct ServerSupervisor {
    server_actor: Option<he_core_core::actors::ActorAddress>,
}

impl ServerSupervisor {
    pub fn new() -> Self {
        Self {
            server_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_core_core::actors::ActorAddress, CoreError> {
        let mut supervisor = he_core_core::actors::ActorSupervisor::new();
        let server_actor = ServerActor::new();
        let address = supervisor.spawn(server_actor);
        
        self.server_actor = Some(address.clone());
        info!("ServerSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_server_actor(&self) -> Option<&he_core_core::actors::ActorAddress> {
        self.server_actor.as_ref()
    }
}

impl Default for ServerSupervisor {
    fn default() -> Self {
        Self::new()
    }
}