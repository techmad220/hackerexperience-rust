//! Client supervisor for managing client processes

use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{ClientError, ClientResult, model::Client};

/// Client supervisor manages client connections and lifecycle
pub struct ClientSupervisor {
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
}

/// Represents a client connection
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub client_id: Uuid,
    pub client_type: Client,
    pub entity_id: Uuid,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl ClientSupervisor {
    /// Create a new client supervisor
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new client connection
    pub async fn register_client(
        &self,
        client_id: Uuid,
        client_type: Client,
        entity_id: Uuid,
    ) -> ClientResult<()> {
        let connection = ClientConnection {
            client_id,
            client_type,
            entity_id,
            connected_at: chrono::Utc::now(),
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id, connection);

        tracing::info!(
            "Client registered: {} (type: {}, entity: {})",
            client_id,
            client_type,
            entity_id
        );

        Ok(())
    }

    /// Unregister a client connection
    pub async fn unregister_client(&self, client_id: &Uuid) -> ClientResult<()> {
        let mut clients = self.clients.write().await;
        
        if clients.remove(client_id).is_some() {
            tracing::info!("Client unregistered: {}", client_id);
            Ok(())
        } else {
            Err(ClientError::ClientNotFound {
                client_id: client_id.to_string(),
            })
        }
    }

    /// Get a client connection
    pub async fn get_client(&self, client_id: &Uuid) -> ClientResult<ClientConnection> {
        let clients = self.clients.read().await;
        
        clients
            .get(client_id)
            .cloned()
            .ok_or_else(|| ClientError::ClientNotFound {
                client_id: client_id.to_string(),
            })
    }

    /// Get all connected clients
    pub async fn get_all_clients(&self) -> Vec<ClientConnection> {
        let clients = self.clients.read().await;
        clients.values().cloned().collect()
    }

    /// Get clients by entity ID
    pub async fn get_clients_by_entity(&self, entity_id: &Uuid) -> Vec<ClientConnection> {
        let clients = self.clients.read().await;
        clients
            .values()
            .filter(|client| &client.entity_id == entity_id)
            .cloned()
            .collect()
    }

    /// Check if client is connected
    pub async fn is_client_connected(&self, client_id: &Uuid) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(client_id)
    }

    /// Get client count
    pub async fn client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }

    /// Get clients by type
    pub async fn get_clients_by_type(&self, client_type: Client) -> Vec<ClientConnection> {
        let clients = self.clients.read().await;
        clients
            .values()
            .filter(|client| client.client_type == client_type)
            .cloned()
            .collect()
    }
}

impl Default for ClientSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_registration() {
        let supervisor = ClientSupervisor::new();
        let client_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        // Register client
        supervisor
            .register_client(client_id, Client::Web1, entity_id)
            .await
            .unwrap();

        // Check client is connected
        assert!(supervisor.is_client_connected(&client_id).await);

        // Get client
        let connection = supervisor.get_client(&client_id).await.unwrap();
        assert_eq!(connection.client_id, client_id);
        assert_eq!(connection.client_type, Client::Web1);
        assert_eq!(connection.entity_id, entity_id);

        // Unregister client
        supervisor.unregister_client(&client_id).await.unwrap();
        assert!(!supervisor.is_client_connected(&client_id).await);
    }

    #[tokio::test]
    async fn test_client_filtering() {
        let supervisor = ClientSupervisor::new();
        let entity_id = Uuid::new_v4();
        let client1_id = Uuid::new_v4();
        let client2_id = Uuid::new_v4();

        // Register clients
        supervisor
            .register_client(client1_id, Client::Web1, entity_id)
            .await
            .unwrap();
        supervisor
            .register_client(client2_id, Client::Mobile1, entity_id)
            .await
            .unwrap();

        // Get clients by entity
        let entity_clients = supervisor.get_clients_by_entity(&entity_id).await;
        assert_eq!(entity_clients.len(), 2);

        // Get clients by type
        let web_clients = supervisor.get_clients_by_type(Client::Web1).await;
        assert_eq!(web_clients.len(), 1);
        assert_eq!(web_clients[0].client_id, client1_id);

        // Check total count
        assert_eq!(supervisor.client_count().await, 2);
    }
}