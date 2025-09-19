use serde::{Deserialize, Serialize};
use crate::{UserId, SoftwareId, IpAddress};

// Placeholder - will be expanded based on PHP SoftwareVPC class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: SoftwareId,
    pub user_id: UserId,
    pub name: String,
    pub software_type: String,
    pub version: String,
    pub size: i32,
    pub location: IpAddress,
}