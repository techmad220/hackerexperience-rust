//! Processable trait and implementations

use crate::types::*;
use async_trait::async_trait;

/// Trait for process behavior and lifecycle management
#[async_trait]
pub trait Processable {
    /// Handle process completion signal
    async fn on_completion(&self, process_id: ProcessId) -> SignalResponse;
    
    /// Handle process pause signal
    async fn on_pause(&self, process_id: ProcessId) -> SignalResponse;
    
    /// Handle process resume signal
    async fn on_resume(&self, process_id: ProcessId) -> SignalResponse;
    
    /// Handle process kill signal
    async fn on_kill(&self, process_id: ProcessId) -> SignalResponse;
    
    /// Handle process update signal
    async fn on_update(&self, process_id: ProcessId, data: serde_json::Value) -> SignalResponse;
    
    /// Handle process checkpoint signal
    async fn on_checkpoint(&self, process_id: ProcessId) -> SignalResponse;
    
    /// Calculate dynamic resource requirements
    async fn calculate_dynamic_resources(&self, process_id: ProcessId) -> Option<DynamicResourceAllocation>;
    
    /// Get process type
    fn process_type(&self) -> ProcessType;
}