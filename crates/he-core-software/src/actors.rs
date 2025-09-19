//! Software Actor System
//!
//! This module provides actor implementations for software and file management,
//! including file operations, software lifecycle, and virus management.

use crate::{FileId, SoftwareId, VirusId, File, Software, Virus, FileType, SoftwareType, StorageId};
use he_core_core::actors::{Actor, ActorContext, Handler, Message};
use he_core_core::CoreError;
use he_core_server::ServerId;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, error, warn, debug};
use serde::{Serialize, Deserialize};

/// Messages for Software Actor
#[derive(Debug)]
pub struct CreateFile {
    pub server_id: ServerId,
    pub file_name: String,
    pub file_type: FileType,
    pub content: Vec<u8>,
    pub path: String,
}

impl Message for CreateFile {
    type Result = Result<File, CoreError>;
}

#[derive(Debug)]
pub struct GetFile {
    pub file_id: FileId,
}

impl Message for GetFile {
    type Result = Result<Option<File>, CoreError>;
}

#[derive(Debug)]
pub struct UpdateFile {
    pub file_id: FileId,
    pub content: Option<Vec<u8>>,
    pub file_name: Option<String>,
}

impl Message for UpdateFile {
    type Result = Result<File, CoreError>;
}

#[derive(Debug)]
pub struct DeleteFile {
    pub file_id: FileId,
}

impl Message for DeleteFile {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct GetServerFiles {
    pub server_id: ServerId,
    pub path: Option<String>,
}

impl Message for GetServerFiles {
    type Result = Result<Vec<File>, CoreError>;
}

#[derive(Debug)]
pub struct CreateSoftware {
    pub server_id: ServerId,
    pub software_type: SoftwareType,
    pub name: String,
    pub version: String,
    pub size: u64,
}

impl Message for CreateSoftware {
    type Result = Result<Software, CoreError>;
}

#[derive(Debug)]
pub struct GetSoftware {
    pub software_id: SoftwareId,
}

impl Message for GetSoftware {
    type Result = Result<Option<Software>, CoreError>;
}

#[derive(Debug)]
pub struct InstallSoftware {
    pub software_id: SoftwareId,
    pub target_server: ServerId,
}

impl Message for InstallSoftware {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct UninstallSoftware {
    pub software_id: SoftwareId,
    pub server_id: ServerId,
}

impl Message for UninstallSoftware {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct GetServerSoftware {
    pub server_id: ServerId,
}

impl Message for GetServerSoftware {
    type Result = Result<Vec<Software>, CoreError>;
}

#[derive(Debug)]
pub struct CreateVirus {
    pub server_id: ServerId,
    pub virus_type: String,
    pub name: String,
    pub power: u32,
    pub size: u64,
}

impl Message for CreateVirus {
    type Result = Result<Virus, CoreError>;
}

#[derive(Debug)]
pub struct GetVirus {
    pub virus_id: VirusId,
}

impl Message for GetVirus {
    type Result = Result<Option<Virus>, CoreError>;
}

#[derive(Debug)]
pub struct InfectServer {
    pub virus_id: VirusId,
    pub target_server: ServerId,
}

impl Message for InfectServer {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct ScanForViruses {
    pub server_id: ServerId,
}

impl Message for ScanForViruses {
    type Result = Result<Vec<Virus>, CoreError>;
}

#[derive(Debug)]
pub struct RemoveVirus {
    pub virus_id: VirusId,
    pub server_id: ServerId,
}

impl Message for RemoveVirus {
    type Result = Result<(), CoreError>;
}

#[derive(Debug)]
pub struct CopyFile {
    pub source_file_id: FileId,
    pub target_server: ServerId,
    pub target_path: String,
}

impl Message for CopyFile {
    type Result = Result<File, CoreError>;
}

#[derive(Debug)]
pub struct MoveFile {
    pub file_id: FileId,
    pub target_path: String,
}

impl Message for MoveFile {
    type Result = Result<File, CoreError>;
}

#[derive(Debug)]
pub struct GetFilesByType {
    pub server_id: ServerId,
    pub file_type: FileType,
}

impl Message for GetFilesByType {
    type Result = Result<Vec<File>, CoreError>;
}

/// Software Actor - manages files, software, and viruses
#[derive(Debug)]
pub struct SoftwareActor {
    /// Files storage (file_id -> File)
    files: Arc<RwLock<HashMap<FileId, File>>>,
    /// Software storage (software_id -> Software)
    software: Arc<RwLock<HashMap<SoftwareId, Software>>>,
    /// Virus storage (virus_id -> Virus)
    viruses: Arc<RwLock<HashMap<VirusId, Virus>>>,
    /// Server to files mapping
    server_files: Arc<RwLock<HashMap<ServerId, Vec<FileId>>>>,
    /// Server to software mapping
    server_software: Arc<RwLock<HashMap<ServerId, Vec<SoftwareId>>>>,
    /// Server to virus mapping (infections)
    server_viruses: Arc<RwLock<HashMap<ServerId, Vec<VirusId>>>>,
}

impl SoftwareActor {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            software: Arc::new(RwLock::new(HashMap::new())),
            viruses: Arc::new(RwLock::new(HashMap::new())),
            server_files: Arc::new(RwLock::new(HashMap::new())),
            server_software: Arc::new(RwLock::new(HashMap::new())),
            server_viruses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new unique file ID
    fn generate_file_id(&self) -> FileId {
        FileId::new()
    }

    /// Generate a new unique software ID
    fn generate_software_id(&self) -> SoftwareId {
        SoftwareId::new()
    }

    /// Generate a new unique virus ID
    fn generate_virus_id(&self) -> VirusId {
        VirusId::new()
    }

    /// Add file to server mapping
    async fn add_file_to_server(&self, server_id: ServerId, file_id: FileId) {
        let mut server_files = self.server_files.write().await;
        server_files.entry(server_id).or_insert_with(Vec::new).push(file_id);
    }

    /// Remove file from server mapping
    async fn remove_file_from_server(&self, server_id: &ServerId, file_id: &FileId) {
        let mut server_files = self.server_files.write().await;
        if let Some(files) = server_files.get_mut(server_id) {
            files.retain(|id| id != file_id);
        }
    }

    /// Add software to server mapping
    async fn add_software_to_server(&self, server_id: ServerId, software_id: SoftwareId) {
        let mut server_software = self.server_software.write().await;
        server_software.entry(server_id).or_insert_with(Vec::new).push(software_id);
    }

    /// Remove software from server mapping
    async fn remove_software_from_server(&self, server_id: &ServerId, software_id: &SoftwareId) {
        let mut server_software = self.server_software.write().await;
        if let Some(software) = server_software.get_mut(server_id) {
            software.retain(|id| id != software_id);
        }
    }

    /// Add virus to server mapping
    async fn add_virus_to_server(&self, server_id: ServerId, virus_id: VirusId) {
        let mut server_viruses = self.server_viruses.write().await;
        server_viruses.entry(server_id).or_insert_with(Vec::new).push(virus_id);
    }

    /// Remove virus from server mapping
    async fn remove_virus_from_server(&self, server_id: &ServerId, virus_id: &VirusId) {
        let mut server_viruses = self.server_viruses.write().await;
        if let Some(viruses) = server_viruses.get_mut(server_id) {
            viruses.retain(|id| id != virus_id);
        }
    }

    /// Calculate file checksum
    fn calculate_checksum(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Validate file path
    fn validate_path(&self, path: &str) -> Result<(), CoreError> {
        if path.is_empty() || path.starts_with("..") || path.contains("//") {
            return Err(CoreError::validation("Invalid file path"));
        }
        Ok(())
    }

    /// Start background virus monitoring
    async fn start_virus_monitoring(&self) {
        let server_viruses = self.server_viruses.clone();
        let viruses = self.viruses.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Monitor virus activity
                let active_viruses = {
                    let server_viruses_guard = server_viruses.read().await;
                    let viruses_guard = viruses.read().await;
                    
                    let mut active = 0;
                    for virus_list in server_viruses_guard.values() {
                        for virus_id in virus_list {
                            if let Some(virus) = viruses_guard.get(virus_id) {
                                if virus.is_active {
                                    active += 1;
                                }
                            }
                        }
                    }
                    active
                };

                if active_viruses > 0 {
                    debug!("Monitoring {} active viruses", active_viruses);
                }
            }
        });
    }
}

impl Actor for SoftwareActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("SoftwareActor started with process_id: {}", ctx.process_id);
        
        // Start virus monitoring
        let actor = self.clone();
        tokio::spawn(async move {
            actor.start_virus_monitoring().await;
        });
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("SoftwareActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: CoreError, ctx: &mut ActorContext) {
        error!("SoftwareActor error on process_id {}: {}", ctx.process_id, err);
    }
}

impl Clone for SoftwareActor {
    fn clone(&self) -> Self {
        Self {
            files: self.files.clone(),
            software: self.software.clone(),
            viruses: self.viruses.clone(),
            server_files: self.server_files.clone(),
            server_software: self.server_software.clone(),
            server_viruses: self.server_viruses.clone(),
        }
    }
}

#[async_trait]
impl Handler<CreateFile> for SoftwareActor {
    async fn handle(&mut self, msg: CreateFile, _ctx: &mut ActorContext) -> Result<File, CoreError> {
        info!("Creating file '{}' on server {}", msg.file_name, msg.server_id);
        
        self.validate_path(&msg.path)?;

        let mut files = self.files.write().await;
        
        let file_id = self.generate_file_id();
        let now = Utc::now();
        let checksum = self.calculate_checksum(&msg.content);

        let file = File {
            file_id,
            server_id: msg.server_id,
            file_name: msg.file_name,
            file_type: msg.file_type,
            content: msg.content.clone(),
            path: msg.path,
            size: msg.content.len() as u64,
            checksum,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            permissions: 0o644, // Default read/write for owner, read for others
            is_hidden: false,
            is_encrypted: false,
            parent_directory: None,
        };

        // Store file
        files.insert(file_id, file.clone());

        // Update server mapping
        self.add_file_to_server(msg.server_id, file_id).await;

        info!("File created: {} ({})", file.file_name, file_id);
        Ok(file)
    }
}

#[async_trait]
impl Handler<GetFile> for SoftwareActor {
    async fn handle(&mut self, msg: GetFile, _ctx: &mut ActorContext) -> Result<Option<File>, CoreError> {
        let files = self.files.read().await;
        let mut file = files.get(&msg.file_id).cloned();
        
        // Update accessed_at timestamp
        if let Some(ref mut f) = file {
            f.accessed_at = Utc::now();
            // In a real implementation, we would update this in storage
        }
        
        Ok(file)
    }
}

#[async_trait]
impl Handler<UpdateFile> for SoftwareActor {
    async fn handle(&mut self, msg: UpdateFile, _ctx: &mut ActorContext) -> Result<File, CoreError> {
        let mut files = self.files.write().await;
        
        let file = files.get_mut(&msg.file_id)
            .ok_or_else(|| CoreError::not_found("File not found"))?;

        // Update content if provided
        if let Some(content) = msg.content {
            file.content = content;
            file.size = file.content.len() as u64;
            file.checksum = self.calculate_checksum(&file.content);
        }

        // Update file name if provided
        if let Some(file_name) = msg.file_name {
            file.file_name = file_name;
        }

        file.modified_at = Utc::now();

        debug!("File updated: {} ({})", file.file_name, msg.file_id);
        Ok(file.clone())
    }
}

#[async_trait]
impl Handler<DeleteFile> for SoftwareActor {
    async fn handle(&mut self, msg: DeleteFile, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut files = self.files.write().await;
        
        if let Some(file) = files.remove(&msg.file_id) {
            // Remove from server mapping
            self.remove_file_from_server(&file.server_id, &msg.file_id).await;
            
            info!("File deleted: {} ({})", file.file_name, msg.file_id);
            Ok(())
        } else {
            Err(CoreError::not_found("File not found"))
        }
    }
}

#[async_trait]
impl Handler<GetServerFiles> for SoftwareActor {
    async fn handle(&mut self, msg: GetServerFiles, _ctx: &mut ActorContext) -> Result<Vec<File>, CoreError> {
        let files = self.files.read().await;
        let server_files = self.server_files.read().await;
        
        if let Some(file_ids) = server_files.get(&msg.server_id) {
            let mut server_files: Vec<File> = file_ids.iter()
                .filter_map(|id| files.get(id).cloned())
                .collect();

            // Filter by path if specified
            if let Some(path) = msg.path {
                server_files.retain(|f| f.path.starts_with(&path));
            }

            Ok(server_files)
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl Handler<CreateSoftware> for SoftwareActor {
    async fn handle(&mut self, msg: CreateSoftware, _ctx: &mut ActorContext) -> Result<Software, CoreError> {
        info!("Creating software '{}' on server {}", msg.name, msg.server_id);

        let mut software_storage = self.software.write().await;
        
        let software_id = self.generate_software_id();
        let now = Utc::now();

        let software = Software {
            software_id,
            server_id: msg.server_id,
            software_type: msg.software_type,
            name: msg.name,
            version: msg.version,
            size: msg.size,
            power_rating: self.calculate_software_power(&msg.software_type),
            created_at: now,
            installed_at: Some(now),
            last_used: None,
            usage_count: 0,
            is_installed: true,
            is_running: false,
            dependencies: Vec::new(),
            license_key: None,
        };

        // Store software
        software_storage.insert(software_id, software.clone());

        // Update server mapping
        self.add_software_to_server(msg.server_id, software_id).await;

        info!("Software created: {} v{} ({})", software.name, software.version, software_id);
        Ok(software)
    }

    /// Calculate software power rating based on type
    fn calculate_software_power(&self, software_type: &SoftwareType) -> u32 {
        match software_type {
            SoftwareType::Cracker => 100,
            SoftwareType::Scanner => 80,
            SoftwareType::Virus => 90,
            SoftwareType::Exploit => 120,
            SoftwareType::Firewall => 110,
            SoftwareType::Antivirus => 95,
            _ => 50, // Default for other types
        }
    }
}

#[async_trait]
impl Handler<GetSoftware> for SoftwareActor {
    async fn handle(&mut self, msg: GetSoftware, _ctx: &mut ActorContext) -> Result<Option<Software>, CoreError> {
        let software = self.software.read().await;
        Ok(software.get(&msg.software_id).cloned())
    }
}

#[async_trait]
impl Handler<InstallSoftware> for SoftwareActor {
    async fn handle(&mut self, msg: InstallSoftware, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut software_storage = self.software.write().await;
        
        let software = software_storage.get_mut(&msg.software_id)
            .ok_or_else(|| CoreError::not_found("Software not found"))?;

        if software.is_installed {
            return Err(CoreError::validation("Software is already installed"));
        }

        software.is_installed = true;
        software.installed_at = Some(Utc::now());
        software.server_id = msg.target_server;

        // Update server mapping
        self.add_software_to_server(msg.target_server, msg.software_id).await;

        info!("Software installed: {} on server {}", software.name, msg.target_server);
        Ok(())
    }
}

#[async_trait]
impl Handler<UninstallSoftware> for SoftwareActor {
    async fn handle(&mut self, msg: UninstallSoftware, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut software_storage = self.software.write().await;
        
        let software = software_storage.get_mut(&msg.software_id)
            .ok_or_else(|| CoreError::not_found("Software not found"))?;

        if software.server_id != msg.server_id {
            return Err(CoreError::validation("Software is not installed on this server"));
        }

        software.is_installed = false;
        software.is_running = false;
        software.installed_at = None;

        // Remove from server mapping
        self.remove_software_from_server(&msg.server_id, &msg.software_id).await;

        info!("Software uninstalled: {} from server {}", software.name, msg.server_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<GetServerSoftware> for SoftwareActor {
    async fn handle(&mut self, msg: GetServerSoftware, _ctx: &mut ActorContext) -> Result<Vec<Software>, CoreError> {
        let software_storage = self.software.read().await;
        let server_software = self.server_software.read().await;
        
        if let Some(software_ids) = server_software.get(&msg.server_id) {
            Ok(software_ids.iter()
                .filter_map(|id| software_storage.get(id).cloned())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl Handler<CreateVirus> for SoftwareActor {
    async fn handle(&mut self, msg: CreateVirus, _ctx: &mut ActorContext) -> Result<Virus, CoreError> {
        info!("Creating virus '{}' on server {}", msg.name, msg.server_id);

        let mut virus_storage = self.viruses.write().await;
        
        let virus_id = self.generate_virus_id();
        let now = Utc::now();

        let virus = Virus {
            virus_id,
            server_id: msg.server_id,
            virus_type: msg.virus_type,
            name: msg.name,
            power: msg.power,
            size: msg.size,
            created_at: now,
            activated_at: None,
            last_activity: None,
            is_active: false,
            is_detected: false,
            infection_count: 0,
            target_files: Vec::new(),
            payload: Vec::new(),
        };

        // Store virus
        virus_storage.insert(virus_id, virus.clone());

        warn!("Virus created: {} ({})", virus.name, virus_id);
        Ok(virus)
    }
}

#[async_trait]
impl Handler<GetVirus> for SoftwareActor {
    async fn handle(&mut self, msg: GetVirus, _ctx: &mut ActorContext) -> Result<Option<Virus>, CoreError> {
        let viruses = self.viruses.read().await;
        Ok(viruses.get(&msg.virus_id).cloned())
    }
}

#[async_trait]
impl Handler<InfectServer> for SoftwareActor {
    async fn handle(&mut self, msg: InfectServer, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut virus_storage = self.viruses.write().await;
        
        let virus = virus_storage.get_mut(&msg.virus_id)
            .ok_or_else(|| CoreError::not_found("Virus not found"))?;

        virus.is_active = true;
        virus.activated_at = Some(Utc::now());
        virus.last_activity = Some(Utc::now());
        virus.infection_count += 1;

        // Add virus to target server
        self.add_virus_to_server(msg.target_server, msg.virus_id).await;

        warn!("Server {} infected with virus {}", msg.target_server, virus.name);
        Ok(())
    }
}

#[async_trait]
impl Handler<ScanForViruses> for SoftwareActor {
    async fn handle(&mut self, msg: ScanForViruses, _ctx: &mut ActorContext) -> Result<Vec<Virus>, CoreError> {
        let viruses_storage = self.viruses.read().await;
        let server_viruses = self.server_viruses.read().await;
        
        if let Some(virus_ids) = server_viruses.get(&msg.server_id) {
            let detected_viruses: Vec<Virus> = virus_ids.iter()
                .filter_map(|id| viruses_storage.get(id).cloned())
                .collect();

            if !detected_viruses.is_empty() {
                warn!("Detected {} viruses on server {}", detected_viruses.len(), msg.server_id);
            }

            Ok(detected_viruses)
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl Handler<RemoveVirus> for SoftwareActor {
    async fn handle(&mut self, msg: RemoveVirus, _ctx: &mut ActorContext) -> Result<(), CoreError> {
        let mut virus_storage = self.viruses.write().await;
        
        if let Some(virus) = virus_storage.get_mut(&msg.virus_id) {
            virus.is_active = false;
            virus.last_activity = Some(Utc::now());
            
            // Remove from server mapping
            self.remove_virus_from_server(&msg.server_id, &msg.virus_id).await;
            
            info!("Virus {} removed from server {}", virus.name, msg.server_id);
            Ok(())
        } else {
            Err(CoreError::not_found("Virus not found"))
        }
    }
}

#[async_trait]
impl Handler<CopyFile> for SoftwareActor {
    async fn handle(&mut self, msg: CopyFile, _ctx: &mut ActorContext) -> Result<File, CoreError> {
        let files = self.files.read().await;
        
        let source_file = files.get(&msg.source_file_id)
            .ok_or_else(|| CoreError::not_found("Source file not found"))?;

        drop(files);

        // Create a copy of the file
        let copy_msg = CreateFile {
            server_id: msg.target_server,
            file_name: source_file.file_name.clone(),
            file_type: source_file.file_type,
            content: source_file.content.clone(),
            path: msg.target_path,
        };

        self.handle(copy_msg, _ctx).await
    }
}

#[async_trait]
impl Handler<MoveFile> for SoftwareActor {
    async fn handle(&mut self, msg: MoveFile, _ctx: &mut ActorContext) -> Result<File, CoreError> {
        self.validate_path(&msg.target_path)?;

        let mut files = self.files.write().await;
        
        let file = files.get_mut(&msg.file_id)
            .ok_or_else(|| CoreError::not_found("File not found"))?;

        file.path = msg.target_path;
        file.modified_at = Utc::now();

        debug!("File moved: {} to {}", file.file_name, file.path);
        Ok(file.clone())
    }
}

#[async_trait]
impl Handler<GetFilesByType> for SoftwareActor {
    async fn handle(&mut self, msg: GetFilesByType, _ctx: &mut ActorContext) -> Result<Vec<File>, CoreError> {
        let files = self.files.read().await;
        let server_files = self.server_files.read().await;
        
        if let Some(file_ids) = server_files.get(&msg.server_id) {
            Ok(file_ids.iter()
                .filter_map(|id| files.get(id))
                .filter(|f| f.file_type == msg.file_type)
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
}

/// Software Supervisor - manages software actor and provides supervision
#[derive(Debug)]
pub struct SoftwareSupervisor {
    software_actor: Option<he_core_core::actors::ActorAddress>,
}

impl SoftwareSupervisor {
    pub fn new() -> Self {
        Self {
            software_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_core_core::actors::ActorAddress, CoreError> {
        let mut supervisor = he_core_core::actors::ActorSupervisor::new();
        let software_actor = SoftwareActor::new();
        let address = supervisor.spawn(software_actor);
        
        self.software_actor = Some(address.clone());
        info!("SoftwareSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_software_actor(&self) -> Option<&he_core_core::actors::ActorAddress> {
        self.software_actor.as_ref()
    }
}

impl Default for SoftwareSupervisor {
    fn default() -> Self {
        Self::new()
    }
}