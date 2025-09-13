use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use he_core::actors::{Actor, ActorContext, ActorHandle, Message};
use he_core::id::{EntityId, ServerId};

use crate::actions::{LogActions, RecoverResult};
use crate::events::{LogCreatedEvent, LogDeletedEvent, LogModifiedEvent};
use crate::models::{CreateLogParams, Log, LogId, LogIndex, RenderedLogIndex, ReviseLogParams};
use crate::queries::LogQueries;

pub struct LogActor {
    actions: LogActions,
    queries: LogQueries,
}

impl LogActor {
    pub fn new(pool: PgPool) -> Self {
        let actions = LogActions::new(pool.clone());
        let queries = LogQueries::new(pool);
        Self { actions, queries }
    }

    pub fn start(pool: PgPool) -> ActorHandle<Self> {
        let actor = Self::new(pool);
        ActorHandle::new(actor)
    }
}

impl Actor for LogActor {
    type Context = ActorContext<Self>;

    async fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LogActor started");
    }

    async fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("LogActor stopped");
    }
}

// Messages

#[derive(Debug)]
pub struct CreateLog {
    pub params: CreateLogParams,
}

impl Message for CreateLog {
    type Result = Result<(Log, Vec<LogCreatedEvent>)>;
}

#[derive(Debug)]
pub struct ReviseLog {
    pub log: Log,
    pub params: ReviseLogParams,
}

impl Message for ReviseLog {
    type Result = Result<(Log, Vec<LogModifiedEvent>)>;
}

#[derive(Debug)]
pub struct RecoverLog {
    pub log: Log,
}

impl Message for RecoverLog {
    type Result = Result<RecoverResult>;
}

#[derive(Debug)]
pub struct GetLog {
    pub log_id: LogId,
}

impl Message for GetLog {
    type Result = Result<Option<Log>>;
}

#[derive(Debug)]
pub struct GetServerLogs {
    pub server_id: ServerId,
}

impl Message for GetServerLogs {
    type Result = Result<Vec<Log>>;
}

#[derive(Debug)]
pub struct GetServerLogIndex {
    pub server_id: ServerId,
}

impl Message for GetServerLogIndex {
    type Result = Result<Vec<LogIndex>>;
}

#[derive(Debug)]
pub struct GetLogsEditedByEntity {
    pub entity_id: EntityId,
}

impl Message for GetLogsEditedByEntity {
    type Result = Result<Vec<Log>>;
}

#[derive(Debug)]
pub struct FindLogsByMessage {
    pub pattern: String,
}

impl Message for FindLogsByMessage {
    type Result = Result<Vec<Log>>;
}

#[derive(Debug)]
pub struct RenderLogIndex {
    pub index: Vec<LogIndex>,
}

impl Message for RenderLogIndex {
    type Result = Vec<RenderedLogIndex>;
}

// Message handlers

#[async_trait::async_trait]
impl he_core::actors::Handler<CreateLog> for LogActor {
    async fn handle(&mut self, msg: CreateLog, _ctx: &mut Self::Context) -> Result<(Log, Vec<LogCreatedEvent>)> {
        self.actions.create(msg.params).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<ReviseLog> for LogActor {
    async fn handle(&mut self, msg: ReviseLog, _ctx: &mut Self::Context) -> Result<(Log, Vec<LogModifiedEvent>)> {
        self.actions.revise(&msg.log, msg.params).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<RecoverLog> for LogActor {
    async fn handle(&mut self, msg: RecoverLog, _ctx: &mut Self::Context) -> Result<RecoverResult> {
        self.actions.recover(&msg.log).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<GetLog> for LogActor {
    async fn handle(&mut self, msg: GetLog, _ctx: &mut Self::Context) -> Result<Option<Log>> {
        self.queries.get_by_id(msg.log_id).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<GetServerLogs> for LogActor {
    async fn handle(&mut self, msg: GetServerLogs, _ctx: &mut Self::Context) -> Result<Vec<Log>> {
        self.queries.get_logs_on_server(msg.server_id).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<GetServerLogIndex> for LogActor {
    async fn handle(&mut self, msg: GetServerLogIndex, _ctx: &mut Self::Context) -> Result<Vec<LogIndex>> {
        self.actions.get_server_log_index(msg.server_id).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<GetLogsEditedByEntity> for LogActor {
    async fn handle(&mut self, msg: GetLogsEditedByEntity, _ctx: &mut Self::Context) -> Result<Vec<Log>> {
        self.queries.get_logs_edited_by_entity(msg.entity_id).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<FindLogsByMessage> for LogActor {
    async fn handle(&mut self, msg: FindLogsByMessage, _ctx: &mut Self::Context) -> Result<Vec<Log>> {
        self.queries.find_by_message_pattern(&msg.pattern).await
    }
}

#[async_trait::async_trait]
impl he_core::actors::Handler<RenderLogIndex> for LogActor {
    async fn handle(&mut self, msg: RenderLogIndex, _ctx: &mut Self::Context) -> Vec<RenderedLogIndex> {
        LogActions::render_index(msg.index)
    }
}