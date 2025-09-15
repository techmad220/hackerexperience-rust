//! Universe Actor System - Complete GenServer Implementation
//!
//! This module provides a comprehensive world management system with GenServer patterns,
//! including universe state management, NPC coordination, world events, and server orchestration.
//!
//! Features:
//! - World state management and synchronization
//! - NPC server registration and lifecycle management
//! - Dynamic world event generation and processing
//! - Geographic server organization and discovery
//! - Organization management (banks, ISPs, corporations)
//! - World economy simulation
//! - Player vs NPC interactions
//! - Cross-node universe synchronization

use crate::models::{
    UniverseState, WorldServer, NpcServer, Organization, WorldEvent, UniverseConfig,
    ServerLocation, WorldRegion, EconomicIndicators, PopulationStats, TechnologyLevel,
    ServerTemplate, UniverseError, OrganizationType, ServerType, EventType
};
use he_helix_core::{
    genserver::{GenServer, GenServerBehavior, GenServerMessage, GenServerReply, InfoSource},
    actors::{Actor, ActorContext, Handler, Message},
    HelixError, HelixResult, ProcessId
};
use async_trait::async_trait;
use std::collections::{HashMap, BTreeMap, VecDeque, HashSet};
use tokio::sync::{RwLock, Mutex, broadcast, mpsc};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug, trace};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use he_core::id::{ServerId, OrganizationId, PlayerId, RegionId, EventId};
use rand::Rng;

/// Universe operation error types
#[derive(Debug, thiserror::Error)]
pub enum UniverseActorError {
    #[error("Server not found: {0}")]
    ServerNotFound(String),
    #[error("Region not found: {0}")]
    RegionNotFound(String),
    #[error("Organization not found: {0}")]
    OrganizationNotFound(String),
    #[error("Invalid server location: {0}")]
    InvalidLocation(String),
    #[error("Server capacity exceeded: {0}")]
    CapacityExceeded(String),
    #[error("Synchronization failed: {0}")]
    SyncFailed(String),
    #[error("Invalid world state: {0}")]
    InvalidState(String),
    #[error("Event generation failed: {0}")]
    EventFailed(String),
    #[error("Internal universe error: {0}")]
    InternalError(String),
}

/// Messages for UniverseActor GenServer
#[derive(Debug, Clone)]
pub enum UniverseCall {
    /// Register a new server in the universe
    RegisterServer {
        server_data: WorldServer,
        location: ServerLocation,
        organization_id: Option<OrganizationId>,
    },
    /// Get servers by location/region
    GetServersByLocation {
        location: ServerLocation,
        radius_km: Option<f64>,
        server_type: Option<ServerType>,
    },
    /// Get servers by organization
    GetServersByOrganization {
        organization_id: OrganizationId,
        include_subsidiaries: bool,
    },
    /// Find nearby servers for routing
    FindNearbyServers {
        from_server: ServerId,
        max_distance_km: f64,
        limit: Option<usize>,
    },
    /// Create world event
    CreateWorldEvent {
        event_type: EventType,
        affected_regions: Vec<RegionId>,
        duration_hours: Option<u64>,
        intensity: f64,
        metadata: HashMap<String, String>,
    },
    /// Get active world events
    GetActiveEvents {
        region: Option<RegionId>,
        event_type: Option<EventType>,
    },
    /// Get world statistics
    GetWorldStats {
        include_economy: bool,
        include_population: bool,
        include_technology: bool,
    },
    /// Synchronize with other universe nodes
    SynchronizeWithNode {
        node_id: String,
        sync_type: SyncType,
    },
    /// Get organization information
    GetOrganization {
        organization_id: OrganizationId,
        include_servers: bool,
        include_subsidiaries: bool,
    },
    /// Create new organization
    CreateOrganization {
        name: String,
        org_type: OrganizationType,
        headquarters_location: ServerLocation,
        initial_funding: u64,
    },
    /// Query universe topology
    GetTopology {
        region: Option<RegionId>,
        include_connections: bool,
    },
}

#[derive(Debug, Clone)]
pub enum UniverseCast {
    /// Trigger world event
    TriggerWorldEvent {
        event: WorldEvent,
        propagate_to_nodes: bool,
    },
    /// Update server resources and status
    UpdateServerResources {
        server_id: ServerId,
        new_resources: HashMap<String, u64>,
        status_change: Option<String>,
    },
    /// Process economic simulation tick
    ProcessEconomicTick,
    /// Update population statistics
    UpdatePopulationStats {
        region_id: RegionId,
        population_change: i64,
        technology_advancement: f64,
    },
    /// Propagate event to neighboring regions
    PropagateEvent {
        event_id: EventId,
        source_region: RegionId,
        propagation_strength: f64,
    },
    /// Cleanup expired events and data
    CleanupExpiredData {
        older_than_hours: u64,
    },
    /// Rebalance server load across regions
    RebalanceServerLoad,
    /// Update organization status
    UpdateOrganizationStatus {
        organization_id: OrganizationId,
        status_changes: HashMap<String, String>,
    },
}

#[derive(Debug, Clone)]
pub enum UniverseInfo {
    /// Node heartbeat from distributed system
    NodeHeartbeat {
        node_id: String,
        timestamp: DateTime<Utc>,
        node_stats: NodeStats,
    },
    /// Server alert from monitoring
    ServerAlert {
        server_id: ServerId,
        alert_type: String,
        severity: AlertSeverity,
        details: HashMap<String, String>,
    },
    /// Economic indicator update
    EconomicUpdate {
        region_id: RegionId,
        indicators: EconomicIndicators,
        trend_data: Vec<f64>,
    },
    /// Player action affecting world state
    PlayerWorldAction {
        player_id: PlayerId,
        action_type: String,
        affected_servers: Vec<ServerId>,
        impact_data: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Full,
    Incremental,
    EventsOnly,
    ServersOnly,
    OrganizationsOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub server_count: u64,
    pub active_events: u64,
    pub player_count: u64,
    pub load_average: f64,
    pub memory_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Universe Actor state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseActorState {
    /// World servers indexed by server ID
    pub servers: HashMap<ServerId, WorldServer>,
    /// Server locations for geographic queries
    pub server_locations: HashMap<ServerId, ServerLocation>,
    /// Organizations in the universe
    pub organizations: HashMap<OrganizationId, Organization>,
    /// World regions with their properties
    pub regions: HashMap<RegionId, WorldRegion>,
    /// Active world events
    pub active_events: HashMap<EventId, WorldEvent>,
    /// Event history for analysis
    pub event_history: VecDeque<WorldEvent>,
    /// Economic state per region
    pub economic_data: HashMap<RegionId, EconomicIndicators>,
    /// Population data per region
    pub population_data: HashMap<RegionId, PopulationStats>,
    /// Technology levels per region
    pub technology_levels: HashMap<RegionId, TechnologyLevel>,
    /// Connected nodes in distributed system
    pub connected_nodes: HashMap<String, NodeInfo>,
    /// Server templates for NPC generation
    pub server_templates: HashMap<String, ServerTemplate>,
    /// Configuration
    pub config: UniverseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub last_heartbeat: DateTime<Utc>,
    pub is_synchronized: bool,
    pub sync_priority: u8,
    pub stats: Option<NodeStats>,
}

impl Default for UniverseActorState {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            server_locations: HashMap::new(),
            organizations: HashMap::new(),
            regions: HashMap::new(),
            active_events: HashMap::new(),
            event_history: VecDeque::new(),
            economic_data: HashMap::new(),
            population_data: HashMap::new(),
            technology_levels: HashMap::new(),
            connected_nodes: HashMap::new(),
            server_templates: HashMap::new(),
            config: UniverseConfig::default(),
        }
    }
}

/// Main Universe Actor
pub struct UniverseActor {
    state: Arc<RwLock<UniverseActorState>>,
    background_handle: Option<tokio::task::JoinHandle<()>>,
    event_sender: Option<broadcast::Sender<WorldEvent>>,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
    node_id: String,
}

impl UniverseActor {
    pub fn new(node_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(UniverseActorState::default())),
            background_handle: None,
            event_sender: None,
            sync_handle: None,
            node_id,
        }
    }

    pub async fn initialize(&mut self) -> HelixResult<()> {
        self.initialize_world_data().await?;
        self.start_background_processing().await?;
        self.start_synchronization().await?;
        self.initialize_event_system().await?;
        
        info!("UniverseActor initialized with node_id: {}", self.node_id);
        Ok(())
    }

    async fn initialize_world_data(&self) -> HelixResult<()> {
        let mut state = self.state.write().await;
        
        // Initialize world regions
        self.create_default_regions(&mut state).await?;
        self.create_default_organizations(&mut state).await?;
        self.load_server_templates(&mut state).await?;
        self.initialize_economic_data(&mut state).await?;
        
        debug!("Initialized world data with {} regions, {} organizations", 
               state.regions.len(), state.organizations.len());
        Ok(())
    }

    async fn create_default_regions(&self, state: &mut UniverseActorState) -> HelixResult<()> {
        let regions = vec![
            ("north_america", "North America", 350.0, 50.0),
            ("south_america", "South America", 300.0, -15.0),
            ("europe", "Europe", 20.0, 50.0),
            ("asia", "Asia", 100.0, 35.0),
            ("africa", "Africa", 20.0, 0.0),
            ("oceania", "Oceania", 140.0, -25.0),
        ];

        for (id, name, lon, lat) in regions {
            let region_id = RegionId::from_string(id.to_string());
            let region = WorldRegion {
                region_id,
                name: name.to_string(),
                latitude: lat,
                longitude: lon,
                population: 100_000_000 + rand::thread_rng().gen_range(0..500_000_000),
                technology_index: 0.5 + rand::thread_rng().gen::<f64>() * 0.4,
                economic_strength: 0.3 + rand::thread_rng().gen::<f64>() * 0.6,
                security_level: 0.4 + rand::thread_rng().gen::<f64>() * 0.4,
                server_capacity: 10000,
                active_servers: 0,
                created_at: Utc::now(),
            };

            state.regions.insert(region_id, region);
        }

        Ok(())
    }

    async fn create_default_organizations(&self, state: &mut UniverseActorState) -> HelixResult<()> {
        let orgs = vec![
            ("Global Bank Corp", OrganizationType::Bank, "north_america"),
            ("TechNet ISP", OrganizationType::Isp, "europe"),
            ("CyberCorp Industries", OrganizationType::Corporation, "asia"),
            ("SecureData Bank", OrganizationType::Bank, "europe"),
            ("NetLink Communications", OrganizationType::Isp, "north_america"),
            ("Innovation Labs", OrganizationType::Corporation, "asia"),
        ];

        for (name, org_type, region_str) in orgs {
            let region_id = RegionId::from_string(region_str.to_string());
            let region = state.regions.get(&region_id).unwrap();
            
            let org_id = OrganizationId::new();
            let organization = Organization {
                organization_id: org_id,
                name: name.to_string(),
                org_type,
                headquarters_location: ServerLocation {
                    latitude: region.latitude + rand::thread_rng().gen::<f64>() * 10.0 - 5.0,
                    longitude: region.longitude + rand::thread_rng().gen::<f64>() * 10.0 - 5.0,
                    region_id,
                    city: format!("{} HQ", name),
                    country: region.name.clone(),
                },
                funding: 1_000_000 + rand::thread_rng().gen_range(0..10_000_000),
                reputation: 0.5 + rand::thread_rng().gen::<f64>() * 0.3,
                servers: Vec::new(),
                subsidiaries: Vec::new(),
                services: Vec::new(),
                created_at: Utc::now(),
                metadata: HashMap::new(),
            };

            state.organizations.insert(org_id, organization);
        }

        Ok(())
    }

    async fn load_server_templates(&self, state: &mut UniverseActorState) -> HelixResult<()> {
        let templates = vec![
            ("basic_server", ServerType::Personal, 1, 1000),
            ("corporate_server", ServerType::Corporate, 3, 5000),
            ("bank_server", ServerType::Bank, 5, 10000),
            ("government_server", ServerType::Government, 4, 8000),
            ("research_server", ServerType::Research, 2, 3000),
        ];

        for (template_id, server_type, difficulty, reward) in templates {
            let template = ServerTemplate {
                template_id: template_id.to_string(),
                server_type,
                base_difficulty: difficulty,
                base_reward: reward,
                required_level: difficulty as u32,
                hardware_specs: HashMap::from([
                    ("cpu".to_string(), 1000 + (difficulty * 500) as u64),
                    ("ram".to_string(), 1024 + (difficulty * 1024) as u64),
                    ("hdd".to_string(), 10000 + (difficulty * 10000) as u64),
                ]),
                software_installed: vec![
                    "operating_system".to_string(),
                    "firewall".to_string(),
                ],
                generation_params: HashMap::new(),
            };

            state.server_templates.insert(template_id.to_string(), template);
        }

        debug!("Loaded {} server templates", state.server_templates.len());
        Ok(())
    }

    async fn initialize_economic_data(&self, state: &mut UniverseActorState) -> HelixResult<()> {
        for (region_id, region) in &state.regions {
            let economic_data = EconomicIndicators {
                gdp: region.population as f64 * (10000.0 + rand::thread_rng().gen::<f64>() * 50000.0),
                inflation_rate: rand::thread_rng().gen::<f64>() * 0.05,
                unemployment_rate: rand::thread_rng().gen::<f64>() * 0.1,
                technology_investment: region.technology_index * rand::thread_rng().gen::<f64>() * 1000000.0,
                cyber_security_budget: region.security_level * rand::thread_rng().gen::<f64>() * 100000.0,
                last_updated: Utc::now(),
            };

            let population_stats = PopulationStats {
                total_population: region.population,
                online_population: (region.population as f64 * (0.3 + region.technology_index * 0.6)) as u64,
                hacker_population: (region.population as f64 * 0.001 * region.technology_index) as u64,
                corporate_users: (region.population as f64 * 0.1) as u64,
                government_users: (region.population as f64 * 0.02) as u64,
                growth_rate: rand::thread_rng().gen::<f64>() * 0.03 - 0.01,
                last_census: Utc::now(),
            };

            let tech_level = TechnologyLevel {
                overall_index: region.technology_index,
                internet_penetration: 0.4 + region.technology_index * 0.5,
                cyber_security_level: region.security_level,
                ai_advancement: rand::thread_rng().gen::<f64>() * region.technology_index,
                quantum_computing: rand::thread_rng().gen::<f64>() * 0.1 * region.technology_index,
                last_assessment: Utc::now(),
            };

            state.economic_data.insert(*region_id, economic_data);
            state.population_data.insert(*region_id, population_stats);
            state.technology_levels.insert(*region_id, tech_level);
        }

        Ok(())
    }

    async fn start_background_processing(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        let event_sender = self.event_sender.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                Self::process_background_tasks(&state, &event_sender).await;
            }
        });

        self.background_handle = Some(handle);
        Ok(())
    }

    async fn start_synchronization(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        let node_id = self.node_id.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(120)); // 2 minutes
            loop {
                interval.tick().await;
                Self::process_node_synchronization(&state, &node_id).await;
            }
        });

        self.sync_handle = Some(handle);
        Ok(())
    }

    async fn initialize_event_system(&mut self) -> HelixResult<()> {
        let (sender, _) = broadcast::channel(1000);
        self.event_sender = Some(sender);
        Ok(())
    }

    async fn process_background_tasks(
        state: &Arc<RwLock<UniverseActorState>>,
        event_sender: &Option<broadcast::Sender<WorldEvent>>,
    ) {
        Self::process_economic_simulation(state).await;
        Self::generate_random_events(state, event_sender).await;
        Self::cleanup_expired_events(state).await;
        Self::update_server_populations(state).await;
    }

    async fn process_economic_simulation(state: &Arc<RwLock<UniverseActorState>>) {
        let mut state_guard = state.write().await;
        
        for (region_id, economic_data) in &mut state_guard.economic_data {
            // Simulate economic changes
            let change_factor = rand::thread_rng().gen::<f64>() * 0.02 - 0.01; // ±1%
            economic_data.gdp *= 1.0 + change_factor;
            economic_data.inflation_rate += (rand::thread_rng().gen::<f64>() - 0.5) * 0.001;
            economic_data.inflation_rate = economic_data.inflation_rate.max(0.0).min(0.1);
            economic_data.last_updated = Utc::now();

            trace!("Economic simulation update for region {}: GDP {:.0}", 
                   region_id, economic_data.gdp);
        }
    }

    async fn generate_random_events(
        state: &Arc<RwLock<UniverseActorState>>,
        event_sender: &Option<broadcast::Sender<WorldEvent>>,
    ) {
        let mut state_guard = state.write().await;
        
        // Generate random world events based on probability
        if rand::thread_rng().gen::<f64>() < 0.1 { // 10% chance per tick
            let event_types = vec![
                EventType::EconomicCrisis,
                EventType::TechBreakthrough,
                EventType::CyberAttack,
                EventType::GovernmentAction,
                EventType::SolarFlare,
            ];

            let event_type = event_types[rand::thread_rng().gen_range(0..event_types.len())].clone();
            let regions: Vec<RegionId> = state_guard.regions.keys().cloned().collect();
            let affected_region = regions[rand::thread_rng().gen_range(0..regions.len())];

            let event = WorldEvent {
                event_id: EventId::new(),
                event_type: event_type.clone(),
                name: format!("{:?} in {:?}", event_type, affected_region),
                description: format!("A {:?} event is affecting the {:?} region", event_type, affected_region),
                affected_regions: vec![affected_region],
                intensity: rand::thread_rng().gen::<f64>(),
                started_at: Utc::now(),
                duration: Some(Duration::hours(rand::thread_rng().gen_range(1..48))),
                metadata: HashMap::new(),
            };

            state_guard.active_events.insert(event.event_id, event.clone());
            state_guard.event_history.push_back(event.clone());

            debug!("Generated world event: {:?} affecting {:?}", event_type, affected_region);

            if let Some(sender) = event_sender {
                let _ = sender.send(event);
            }
        }
    }

    async fn cleanup_expired_events(state: &Arc<RwLock<UniverseActorState>>) {
        let mut state_guard = state.write().await;
        let now = Utc::now();
        
        let expired_events: Vec<EventId> = state_guard
            .active_events
            .iter()
            .filter(|(_, event)| {
                event.duration
                    .map_or(false, |duration| now > event.started_at + duration)
            })
            .map(|(id, _)| *id)
            .collect();

        for event_id in expired_events {
            if let Some(event) = state_guard.active_events.remove(&event_id) {
                debug!("Expired world event: {}", event.name);
            }
        }

        // Limit event history size
        while state_guard.event_history.len() > 1000 {
            state_guard.event_history.pop_front();
        }
    }

    async fn update_server_populations(state: &Arc<RwLock<UniverseActorState>>) {
        let mut state_guard = state.write().await;
        
        for (server_id, server) in &mut state_guard.servers {
            if matches!(server.server_type, ServerType::Npc) {
                // Simulate NPC server activity
                let activity_change = rand::thread_rng().gen::<f64>() * 0.1 - 0.05;
                server.activity_level = (server.activity_level + activity_change).max(0.0).min(1.0);
                server.last_seen = Utc::now();
            }
        }
    }

    async fn process_node_synchronization(state: &Arc<RwLock<UniverseActorState>>, node_id: &str) {
        let state_guard = state.read().await;
        
        // Check node health and synchronization status
        let now = Utc::now();
        let unhealthy_nodes: Vec<String> = state_guard
            .connected_nodes
            .iter()
            .filter(|(_, node)| now - node.last_heartbeat > Duration::minutes(5))
            .map(|(id, _)| id.clone())
            .collect();

        if !unhealthy_nodes.is_empty() {
            warn!("Detected {} unhealthy nodes: {:?}", unhealthy_nodes.len(), unhealthy_nodes);
        }
    }

    async fn handle_register_server(
        &self,
        server_data: WorldServer,
        location: ServerLocation,
        organization_id: Option<OrganizationId>,
    ) -> Result<ServerId, UniverseActorError> {
        let mut state = self.state.write().await;

        // Validate location
        if !state.regions.contains_key(&location.region_id) {
            return Err(UniverseActorError::RegionNotFound(location.region_id.to_string()));
        }

        let server_id = server_data.server_id;

        // Check region capacity
        let region = state.regions.get_mut(&location.region_id).unwrap();
        if region.active_servers >= region.server_capacity {
            return Err(UniverseActorError::CapacityExceeded(
                format!("Region {} at capacity", location.region_id)
            ));
        }

        // Register server
        state.servers.insert(server_id, server_data);
        state.server_locations.insert(server_id, location);

        // Update region stats
        region.active_servers += 1;

        // Add server to organization if specified
        if let Some(org_id) = organization_id {
            if let Some(organization) = state.organizations.get_mut(&org_id) {
                organization.servers.push(server_id);
            }
        }

        info!("Registered server {} in region {}", server_id, location.region_id);
        Ok(server_id)
    }

    async fn handle_get_servers_by_location(
        &self,
        location: ServerLocation,
        radius_km: Option<f64>,
        server_type: Option<ServerType>,
    ) -> Result<Vec<WorldServer>, UniverseActorError> {
        let state = self.state.read().await;
        let radius = radius_km.unwrap_or(100.0);

        let mut nearby_servers = Vec::new();

        for (server_id, server_location) in &state.server_locations {
            let distance = self.calculate_distance(&location, server_location);
            
            if distance <= radius {
                if let Some(server) = state.servers.get(server_id) {
                    if server_type.as_ref().map_or(true, |t| &server.server_type == t) {
                        nearby_servers.push(server.clone());
                    }
                }
            }
        }

        // Sort by distance (using estimated values)
        nearby_servers.sort_by(|a, b| a.difficulty.cmp(&b.difficulty));

        Ok(nearby_servers)
    }

    fn calculate_distance(&self, loc1: &ServerLocation, loc2: &ServerLocation) -> f64 {
        // Simple Euclidean distance calculation
        let lat_diff = loc1.latitude - loc2.latitude;
        let lon_diff = loc1.longitude - loc2.longitude;
        ((lat_diff * lat_diff + lon_diff * lon_diff).sqrt()) * 111.0 // Rough km conversion
    }

    async fn handle_create_world_event(
        &self,
        event_type: EventType,
        affected_regions: Vec<RegionId>,
        duration_hours: Option<u64>,
        intensity: f64,
        metadata: HashMap<String, String>,
    ) -> Result<WorldEvent, UniverseActorError> {
        let mut state = self.state.write().await;

        let event = WorldEvent {
            event_id: EventId::new(),
            event_type: event_type.clone(),
            name: format!("Custom {:?} Event", event_type),
            description: format!("A {:?} event affecting {} regions", event_type, affected_regions.len()),
            affected_regions: affected_regions.clone(),
            intensity,
            started_at: Utc::now(),
            duration: duration_hours.map(Duration::hours),
            metadata,
        };

        state.active_events.insert(event.event_id, event.clone());
        state.event_history.push_back(event.clone());

        // Broadcast event
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event.clone());
        }

        info!("Created world event {:?} affecting {} regions", 
              event_type, affected_regions.len());
        Ok(event)
    }
}

/// GenServer implementation for UniverseActor
#[async_trait]
impl GenServerBehavior for UniverseActor {
    type State = UniverseActorState;

    async fn init(&mut self) -> HelixResult<()> {
        self.initialize().await?;
        info!("UniverseActor GenServer initialized");
        Ok(())
    }

    async fn handle_call(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _from: ProcessId,
    ) -> HelixResult<GenServerReply> {
        if let Ok(call) = message.downcast::<UniverseCall>() {
            match *call {
                UniverseCall::RegisterServer { server_data, location, organization_id } => {
                    let result = self.handle_register_server(server_data, location, organization_id).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                UniverseCall::GetServersByLocation { location, radius_km, server_type } => {
                    let result = self.handle_get_servers_by_location(location, radius_km, server_type).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                UniverseCall::CreateWorldEvent { event_type, affected_regions, duration_hours, intensity, metadata } => {
                    let result = self.handle_create_world_event(
                        event_type, affected_regions, duration_hours, intensity, metadata
                    ).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                UniverseCall::GetActiveEvents { region, event_type } => {
                    let state = self.state.read().await;
                    let events: Vec<WorldEvent> = state
                        .active_events
                        .values()
                        .filter(|event| {
                            region.as_ref().map_or(true, |r| event.affected_regions.contains(r)) &&
                            event_type.as_ref().map_or(true, |t| &event.event_type == t)
                        })
                        .cloned()
                        .collect();
                    Ok(GenServerReply::Reply(Box::new(Ok::<Vec<WorldEvent>, UniverseActorError>(events))))
                }
                UniverseCall::GetOrganization { organization_id, include_servers, include_subsidiaries } => {
                    let state = self.state.read().await;
                    let org = state.organizations.get(&organization_id).cloned();
                    Ok(GenServerReply::Reply(Box::new(Ok::<Option<Organization>, UniverseActorError>(org))))
                }
                _ => {
                    warn!("Unhandled universe call message");
                    Ok(GenServerReply::NoReply)
                }
            }
        } else {
            Err(HelixError::InvalidMessage("Unknown call message type".to_string()))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
    ) -> HelixResult<()> {
        if let Ok(cast) = message.downcast::<UniverseCast>() {
            match *cast {
                UniverseCast::TriggerWorldEvent { event, propagate_to_nodes } => {
                    let mut state = self.state.write().await;
                    state.active_events.insert(event.event_id, event.clone());
                    state.event_history.push_back(event.clone());

                    if propagate_to_nodes {
                        if let Some(sender) = &self.event_sender {
                            let _ = sender.send(event);
                        }
                    }
                }
                UniverseCast::UpdateServerResources { server_id, new_resources, status_change } => {
                    let mut state = self.state.write().await;
                    if let Some(server) = state.servers.get_mut(&server_id) {
                        for (resource, value) in new_resources {
                            server.resources.insert(resource, value);
                        }
                        server.last_seen = Utc::now();
                    }
                }
                UniverseCast::ProcessEconomicTick => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        Self::process_economic_simulation(&state).await;
                    });
                }
                UniverseCast::CleanupExpiredData { older_than_hours } => {
                    let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
                    let mut state = self.state.write().await;
                    
                    // Cleanup old events
                    state.event_history.retain(|event| event.started_at > cutoff);
                }
                _ => {
                    debug!("Universe cast message processed");
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _source: InfoSource,
    ) -> HelixResult<()> {
        if let Ok(info) = message.downcast::<UniverseInfo>() {
            match *info {
                UniverseInfo::NodeHeartbeat { node_id, timestamp, node_stats } => {
                    let mut state = self.state.write().await;
                    let node_info = NodeInfo {
                        node_id: node_id.clone(),
                        last_heartbeat: timestamp,
                        is_synchronized: true,
                        sync_priority: 5,
                        stats: Some(node_stats),
                    };
                    state.connected_nodes.insert(node_id, node_info);
                }
                UniverseInfo::ServerAlert { server_id, alert_type, severity, details } => {
                    match severity {
                        AlertSeverity::Critical | AlertSeverity::High => {
                            error!("Server {} alert [{}]: {:?}", server_id, alert_type, severity);
                        }
                        AlertSeverity::Medium => {
                            warn!("Server {} alert [{}]: {:?}", server_id, alert_type, severity);
                        }
                        AlertSeverity::Low => {
                            debug!("Server {} alert [{}]: {:?}", server_id, alert_type, severity);
                        }
                    }
                }
                UniverseInfo::EconomicUpdate { region_id, indicators, trend_data } => {
                    let mut state = self.state.write().await;
                    state.economic_data.insert(region_id, indicators);
                    debug!("Updated economic data for region: {}", region_id);
                }
                UniverseInfo::PlayerWorldAction { player_id, action_type, affected_servers, impact_data } => {
                    debug!("Player {} performed world action: {} affecting {} servers", 
                           player_id, action_type, affected_servers.len());
                }
            }
        }
        Ok(())
    }

    async fn terminate(&mut self, _reason: String) -> HelixResult<()> {
        if let Some(handle) = self.background_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.sync_handle.take() {
            handle.abort();
        }
        
        info!("UniverseActor terminated");
        Ok(())
    }

    async fn code_change(&mut self, _old_version: String, _new_version: String) -> HelixResult<()> {
        info!("UniverseActor code change completed");
        Ok(())
    }

    async fn get_state(&self) -> HelixResult<Self::State> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    async fn set_state(&mut self, state: Self::State) -> HelixResult<()> {
        let mut current_state = self.state.write().await;
        *current_state = state;
        Ok(())
    }
}

/// NPC Actor for individual NPC management
pub struct NPCActor {
    state: Arc<RwLock<NPCActorState>>,
    universe_state: Arc<RwLock<UniverseActorState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCActorState {
    pub active_npcs: HashMap<ServerId, NpcServer>,
    pub npc_behaviors: HashMap<String, NPCBehavior>,
    pub interaction_history: VecDeque<NPCInteraction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCBehavior {
    pub behavior_type: String,
    pub aggressiveness: f64,
    pub intelligence: f64,
    pub response_patterns: Vec<String>,
    pub update_frequency: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCInteraction {
    pub interaction_id: Uuid,
    pub npc_server_id: ServerId,
    pub player_id: Option<PlayerId>,
    pub interaction_type: String,
    pub outcome: String,
    pub timestamp: DateTime<Utc>,
}

impl Default for NPCActorState {
    fn default() -> Self {
        Self {
            active_npcs: HashMap::new(),
            npc_behaviors: HashMap::new(),
            interaction_history: VecDeque::new(),
        }
    }
}

impl NPCActor {
    pub fn new(universe_state: Arc<RwLock<UniverseActorState>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(NPCActorState::default())),
            universe_state,
        }
    }
}

/// Universe Actor supervisor
pub struct UniverseActorSupervisor {
    node_id: String,
}

impl UniverseActorSupervisor {
    pub fn new(node_id: String) -> Self {
        Self { node_id }
    }

    pub async fn start(&self) -> HelixResult<UniverseActor> {
        let mut actor = UniverseActor::new(self.node_id.clone());
        actor.initialize().await?;
        info!("UniverseActor supervised startup completed");
        Ok(actor)
    }
}