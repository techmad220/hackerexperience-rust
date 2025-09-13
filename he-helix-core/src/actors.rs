//! Actor system foundation for Helix
//!
//! This module provides the basic actor model infrastructure inspired by Elixir/OTP
//! but adapted for Rust and Tokio.

use crate::{HelixError, HelixResult, ProcessId, RequestId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Base trait for all actor messages
pub trait Message: Send + Sync + Debug + 'static {
    /// The type returned when this message is processed
    type Result: Send + 'static;
}

/// Trait for actor message handlers
#[async_trait]
pub trait Handler<M: Message>: Actor {
    /// Handle the incoming message
    async fn handle(&mut self, msg: M, ctx: &mut ActorContext) -> M::Result;
}

/// Base trait for all actors
pub trait Actor: Send + 'static {
    /// Called when the actor is started
    fn started(&mut self, _ctx: &mut ActorContext) {}

    /// Called when the actor is about to be stopped
    fn stopping(&mut self, _ctx: &mut ActorContext) {}

    /// Called when the actor encounters an error
    fn error(&mut self, err: HelixError, _ctx: &mut ActorContext) {
        tracing::error!("Actor error: {}", err);
    }
}

/// Actor execution context
pub struct ActorContext {
    /// Unique identifier for this actor process
    pub process_id: ProcessId,
    /// Request ID if this actor was started as part of a request
    pub request_id: Option<RequestId>,
    /// Actor's mailbox sender
    pub address: ActorAddress,
    /// Shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Whether the actor should stop
    should_stop: bool,
}

impl ActorContext {
    pub fn new(address: ActorAddress) -> Self {
        Self {
            process_id: ProcessId::new(),
            request_id: None,
            address,
            shutdown_tx: None,
            should_stop: false,
        }
    }

    /// Stop the actor
    pub fn stop(&mut self) {
        self.should_stop = true;
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Check if the actor should stop
    pub fn should_stop(&self) -> bool {
        self.should_stop
    }

    /// Set the request ID for this actor
    pub fn set_request_id(&mut self, request_id: RequestId) {
        self.request_id = Some(request_id);
    }
}

/// Address for sending messages to an actor
#[derive(Clone)]
pub struct ActorAddress {
    tx: mpsc::UnboundedSender<Box<dyn ActorMessage>>,
}

impl ActorAddress {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Box<dyn ActorMessage>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }

    /// Send a message to the actor
    pub async fn send<M>(&self, msg: M) -> HelixResult<M::Result>
    where
        M: Message + 'static,
    {
        let (response_tx, response_rx) = oneshot::channel();
        let envelope = MessageEnvelope {
            message: Box::new(msg),
            response_tx: Some(Box::new(response_tx)),
        };

        self.tx
            .send(Box::new(envelope))
            .map_err(|_| HelixError::actor("Failed to send message - actor may be stopped"))?;

        response_rx
            .await
            .map_err(|_| HelixError::actor("Failed to receive response from actor"))?
            .downcast::<M::Result>()
            .map(|boxed| *boxed)
            .map_err(|_| HelixError::actor("Failed to downcast response"))
    }

    /// Try to send a message without waiting for response
    pub fn try_send<M>(&self, msg: M) -> HelixResult<()>
    where
        M: Message + 'static,
    {
        let envelope = MessageEnvelope {
            message: Box::new(msg),
            response_tx: None,
        };

        self.tx
            .send(Box::new(envelope))
            .map_err(|_| HelixError::actor("Failed to send message - actor may be stopped"))
    }
}

/// Trait for boxed actor messages
trait ActorMessage: Send + 'static {
    fn handle(&mut self, actor: &mut dyn Any, ctx: &mut ActorContext) -> HelixResult<()>;
}

/// Message envelope for actor communication
struct MessageEnvelope<M: Message> {
    message: Box<M>,
    response_tx: Option<Box<dyn ResponseSender<M::Result>>>,
}

impl<M: Message + 'static> ActorMessage for MessageEnvelope<M> {
    fn handle(&mut self, actor: &mut dyn Any, ctx: &mut ActorContext) -> HelixResult<()> {
        // This is a simplified implementation - in practice, we'd need more sophisticated
        // message handling with proper type erasure and downcasting
        Ok(())
    }
}

/// Trait for response senders
trait ResponseSender<T>: Send {
    fn send(self: Box<Self>, response: T) -> Result<(), T>;
}

impl<T: Send + 'static> ResponseSender<T> for oneshot::Sender<Box<dyn Any + Send>> {
    fn send(self: Box<Self>, response: T) -> Result<(), T> {
        (*self)
            .send(Box::new(response))
            .map_err(|boxed| *boxed.downcast().unwrap())
    }
}

/// System message for stopping an actor
#[derive(Debug)]
pub struct Stop;

impl Message for Stop {
    type Result = ();
}

/// System message for pinging an actor
#[derive(Debug)]
pub struct Ping;

impl Message for Ping {
    type Result = Pong;
}

#[derive(Debug)]
pub struct Pong;

/// Actor supervisor for managing actor lifecycles
pub struct ActorSupervisor {
    actors: Vec<ActorHandle>,
}

impl ActorSupervisor {
    pub fn new() -> Self {
        Self {
            actors: Vec::new(),
        }
    }

    /// Spawn a new actor under this supervisor
    pub fn spawn<A: Actor + 'static>(&mut self, actor: A) -> ActorAddress {
        let (address, rx) = ActorAddress::new();
        let handle = ActorHandle::spawn(actor, address.clone(), rx);
        self.actors.push(handle);
        address
    }

    /// Stop all actors under this supervisor
    pub async fn stop_all(&mut self) -> HelixResult<()> {
        for handle in self.actors.drain(..) {
            handle.stop().await?;
        }
        Ok(())
    }
}

/// Handle for a running actor
pub struct ActorHandle {
    process_id: ProcessId,
    shutdown_tx: oneshot::Sender<()>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl ActorHandle {
    fn spawn<A: Actor + 'static>(
        mut actor: A,
        address: ActorAddress,
        mut rx: mpsc::UnboundedReceiver<Box<dyn ActorMessage>>,
    ) -> Self {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let process_id = ProcessId::new();
        let mut ctx = ActorContext::new(address);
        ctx.process_id = process_id;

        let join_handle = tokio::spawn(async move {
            actor.started(&mut ctx);

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(mut msg) => {
                                if let Err(err) = msg.handle(&mut actor as &mut dyn Any, &mut ctx) {
                                    actor.error(err, &mut ctx);
                                }
                            }
                            None => break, // Channel closed
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }

                if ctx.should_stop() {
                    break;
                }
            }

            actor.stopping(&mut ctx);
        });

        Self {
            process_id,
            shutdown_tx,
            join_handle,
        }
    }

    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    pub async fn stop(self) -> HelixResult<()> {
        let _ = self.shutdown_tx.send(());
        self.join_handle
            .await
            .map_err(|e| HelixError::actor(format!("Failed to stop actor: {}", e)))?;
        Ok(())
    }
}