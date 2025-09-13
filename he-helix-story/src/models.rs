use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use he_core::id::{EntityId, ServerId};

pub type StepName = String;
pub type EmailId = String;
pub type ReplyId = String;
pub type ContactId = String;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Story {
    pub story_id: Uuid,
    pub entity_id: EntityId,
    pub current_step: StepName,
    pub meta: serde_json::Value,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: StepName,
    pub entity_id: EntityId,
    pub meta: HashMap<String, serde_json::Value>,
    pub contact: Option<ContactId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub name: String,
    pub steps: Vec<StepName>,
    pub current_step: StepName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: EmailId,
    pub replies: Vec<ReplyId>,
    pub locked: Vec<ReplyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub id: ReplyId,
    pub email_id: EmailId,
    pub content: String,
    pub unlocked: bool,
}

#[derive(Debug, Clone)]
pub enum CallbackAction {
    Complete,
    CompleteWithOptions(Vec<String>),
    Restart { reason: String, checkpoint: EmailId },
    SendEmail { email_id: EmailId, meta: HashMap<String, serde_json::Value>, options: Vec<String> },
    SendReply { reply_id: ReplyId, options: Vec<String> },
    NoOp,
}

#[derive(Debug, Clone)]
pub struct CreateStoryParams {
    pub entity_id: EntityId,
    pub initial_step: Option<StepName>,
}

#[derive(Debug, Clone)]
pub struct UpdateStoryParams {
    pub current_step: Option<StepName>,
    pub meta: Option<serde_json::Value>,
}