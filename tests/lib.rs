pub mod common;
pub mod fixtures;
pub mod test_infrastructure;

// Unit tests
pub mod unit {
    pub mod test_ajax_handlers;
    pub mod test_actor_systems; 
    pub mod test_infrastructure;
}

// Integration tests
pub mod integration {
    pub mod test_api_endpoints;
    pub mod test_database_integration;
    pub mod test_websocket_communication;
    pub mod test_authentication_flows;
}

// Performance tests
pub mod performance {
    pub mod test_load_performance;
}

// Security tests
pub mod security {
    pub mod test_security_vulnerabilities;
}

pub use common::*;
pub use fixtures::*;
pub use test_infrastructure::*;