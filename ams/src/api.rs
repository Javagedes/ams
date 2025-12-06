//! This module contains the message structs that can be sent and received by clients.
//!
//! AMS supports both peer to peer and client-server architectures. Messages being sent through a server must be
//! wrapped in a `Server` message, while peer to peer messages can be sent directly as is.
use serde_derive::*;

/// A command to send a message to another client.
#[derive(Serialize, Deserialize)]
pub struct Message {
    /// The unique id of the message
    pub id: u64,
    /// The payload
    pub payload: Vec<u8>,
    /// The sender connection id // SocketAddr -> String
    pub sender: String,
}
