//! Client utility functions

use crate::types::ClientNip;
use uuid::Uuid;

/// Convert network ID and IP to ClientNip
pub fn to_nip(network_tuple: (Uuid, String)) -> ClientNip {
    ClientNip {
        network_id: network_tuple.0,
        ip: network_tuple.1,
    }
}