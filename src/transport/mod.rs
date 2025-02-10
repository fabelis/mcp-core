use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::{
    error::McpError,
    protocol::types::*, // Import all JSON-RPC types from protocol
};

pub use sse::ClientTransport as SseClientTransport;
pub use sse::ServerTransport as SseServerTransport;

// Message types for the transport actor
#[derive(Debug)]
pub enum TransportCommand {
    SendMessage(JsonRpcMessage),
    Close,
}

#[derive(Debug)]
pub enum TransportEvent {
    Message(JsonRpcMessage),
    Error(McpError),
    Closed,
}

#[async_trait]
pub trait TransportTrait: Send + Sync + 'static {
    async fn start(&mut self) -> Result<TransportChannels, McpError>;

    fn default() -> Self
    where
        Self: Sized;
}
#[async_trait]
pub trait ServerTransportTrait: TransportTrait {}

#[async_trait]
pub trait ClientTransportTrait: TransportTrait {}

// Channels for communicating with the transport
#[derive(Debug, Clone)]
pub struct TransportChannels {
    /// Send commands to the transport
    pub cmd_tx: mpsc::Sender<TransportCommand>,
    /// Receive events from the transport
    pub event_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<TransportEvent>>>,
}

// Stdio Transport Implementation
pub mod stdio;

// SSE Transport Implementation
pub mod sse;
