//! Plugin system for HackerExperience API
//!
//! This module provides a trait-based plugin system that allows modular
//! registration of API endpoints, middleware, and services.

use actix_web::{web, Scope};
use async_trait::async_trait;
use std::any::Any;

/// Core trait for API plugins
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin
    fn name(&self) -> &'static str;

    /// Returns the version of the plugin
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Returns a description of what this plugin provides
    fn description(&self) -> &'static str {
        "No description provided"
    }

    /// Returns the list of dependencies this plugin requires
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Initialize the plugin (called once at startup)
    async fn initialize(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    /// Register routes for this plugin
    fn register_routes(&self, scope: Scope) -> Scope {
        scope
    }

    /// Register middleware for this plugin
    fn register_middleware(&self, cfg: &mut web::ServiceConfig) {
        // Default: no middleware
    }

    /// Called when the plugin is being shut down
    async fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Returns metadata about the plugin
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name(),
            version: self.version(),
            description: self.description(),
            dependencies: self.dependencies(),
        }
    }
}

/// Plugin context for passing data between plugins
pub struct PluginContext {
    pub app_state: web::Data<crate::AppState>,
    pub shared_data: std::collections::HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(app_state: web::Data<crate::AppState>) -> Self {
        Self {
            app_state,
            shared_data: std::collections::HashMap::new(),
        }
    }

    /// Store shared data for other plugins
    pub fn set_shared<T: Any + Send + Sync + 'static>(&mut self, key: String, value: T) {
        self.shared_data.insert(key, Box::new(value));
    }

    /// Retrieve shared data from another plugin
    pub fn get_shared<T: Any + Send + Sync + 'static>(&self, key: &str) -> Option<&T> {
        self.shared_data
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
    }
}

/// Metadata about a plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub dependencies: Vec<&'static str>,
}

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),

    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Plugin shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("Invalid plugin configuration: {0}")]
    InvalidConfiguration(String),
}

/// Plugin manager for handling plugin lifecycle
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    initialized: bool,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            initialized: false,
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        // Check if plugin is already registered
        if self.plugins.iter().any(|p| p.name() == plugin.name()) {
            return Err(PluginError::AlreadyRegistered(plugin.name().to_string()));
        }

        self.plugins.push(plugin);
        Ok(())
    }

    /// Initialize all plugins
    pub async fn initialize(&mut self, app_state: web::Data<crate::AppState>) -> Result<(), PluginError> {
        if self.initialized {
            return Ok(());
        }

        let mut context = PluginContext::new(app_state);

        // Sort plugins by dependencies
        let sorted = self.sort_by_dependencies()?;

        // Initialize in dependency order
        for plugin in &sorted {
            plugin.initialize(&mut context).await
                .map_err(|e| PluginError::InitializationFailed(
                    format!("{}: {}", plugin.name(), e)
                ))?;
        }

        self.plugins = sorted;
        self.initialized = true;
        Ok(())
    }

    /// Configure all plugin routes
    pub fn configure_routes(&self, cfg: &mut web::ServiceConfig) {
        let mut scope = web::scope("/api/v1");

        for plugin in &self.plugins {
            scope = plugin.register_routes(scope);
        }

        cfg.service(scope);
    }

    /// Configure all plugin middleware
    pub fn configure_middleware(&self, cfg: &mut web::ServiceConfig) {
        for plugin in &self.plugins {
            plugin.register_middleware(cfg);
        }
    }

    /// Shutdown all plugins
    pub async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Shutdown in reverse order
        for plugin in self.plugins.iter().rev() {
            plugin.shutdown().await
                .map_err(|e| PluginError::ShutdownFailed(
                    format!("{}: {}", plugin.name(), e)
                ))?;
        }

        self.initialized = false;
        Ok(())
    }

    /// Get list of loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.iter().map(|p| p.metadata()).collect()
    }

    /// Sort plugins by dependencies (topological sort)
    fn sort_by_dependencies(&self) -> Result<Vec<Box<dyn Plugin>>, PluginError> {
        // Build dependency graph
        let mut graph: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        let mut in_degree: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

        for plugin in &self.plugins {
            let name = plugin.name();
            graph.entry(name).or_insert_with(Vec::new);
            in_degree.entry(name).or_insert(0);

            for dep in plugin.dependencies() {
                // Check dependency exists
                if !self.plugins.iter().any(|p| p.name() == dep) {
                    return Err(PluginError::DependencyNotFound(dep.to_string()));
                }

                graph.entry(dep).or_insert_with(Vec::new).push(name);
                *in_degree.entry(name).or_insert(0) += 1;
            }
        }

        // Topological sort using Kahn's algorithm
        let mut sorted = Vec::new();
        let mut queue: Vec<&str> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&name, _)| name)
            .collect();

        while let Some(current) = queue.pop() {
            // Find the plugin
            let plugin = self.plugins.iter()
                .find(|p| p.name() == current)
                .unwrap();

            sorted.push(plugin.name());

            // Process dependencies
            if let Some(deps) = graph.get(current) {
                for &dep in deps {
                    let degree = in_degree.get_mut(dep).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(dep);
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != self.plugins.len() {
            return Err(PluginError::InitializationFailed(
                "Circular dependency detected in plugins".to_string()
            ));
        }

        // Return plugins in sorted order
        Ok(sorted.iter()
            .map(|&name| {
                self.plugins.iter()
                    .find(|p| p.name() == name)
                    .unwrap()
                    .as_ref()
            })
            .map(|p| {
                // This is a bit hacky but necessary for the trait object
                // In a real implementation, we'd use Arc or similar
                Box::new(PluginProxy::new(p)) as Box<dyn Plugin>
            })
            .collect())
    }
}

/// Proxy struct for cloning plugin references
struct PluginProxy {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    dependencies: Vec<&'static str>,
}

impl PluginProxy {
    fn new(plugin: &dyn Plugin) -> Self {
        Self {
            name: plugin.name(),
            version: plugin.version(),
            description: plugin.description(),
            dependencies: plugin.dependencies(),
        }
    }
}

#[async_trait]
impl Plugin for PluginProxy {
    fn name(&self) -> &'static str {
        self.name
    }

    fn version(&self) -> &'static str {
        self.version
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn dependencies(&self) -> Vec<&'static str> {
        self.dependencies.clone()
    }
}

// Include macros module
#[macro_use]
pub mod macros;

// Re-export common types
pub use self::{Plugin, PluginContext, PluginError, PluginManager, PluginMetadata};