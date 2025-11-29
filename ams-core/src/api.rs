//! This module contains the message structs that can be sent and received by clients.
//!
//! AMS supports both peer to peer and client-server architectures. Messages being sent through a server must be
//! wrapped in a `Server` message, while peer to peer messages can be sent directly as is.
use serde_derive::*;

/// A unique identifier for an active connection to the AMS server.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct ConnectionId {
    /// An opaque numeric identifier for the connection.
    pub id: usize,
    /// A generation counter for the connection. This is incremented when the connection id is reused. This helps to
    /// prevent messages being delivered to a stale connection.
    pub generation: usize,
}

/// A command to send a message to another client.
#[derive(Serialize, Deserialize)]
pub struct Message {
    /// The payload
    pub payload: String,
    /// The sender connection id
    pub sender: ConnectionId,
    /// The recipient connection id
    pub receiver: ConnectionId,
}
