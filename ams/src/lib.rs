#![doc = include_str!("../../README.md")]

pub mod api;
mod connection;
mod connection_manager;
mod controller;
mod layers;

use std::{net::SocketAddr, time::SystemTime};

use tokio::sync::mpsc;

use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use crate::connection_manager::ConnectionManager;

/// The AMS instance.
pub struct Ams {
    /// The connection manager.
    manager: ConnectionManager,
    /// The event stream.
    event_stream: UnboundedReceiverStream<Event>,
}

impl Ams {
    /// Starts up an AMS instance on a task, binding to the specified address.
    pub async fn bind(addr: impl ToString) -> std::io::Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let stream = UnboundedReceiverStream::new(event_rx);

        Ok(Self {
            manager: ConnectionManager::spawn(addr, event_tx).await?,
            event_stream: stream,
        })
    }

    /// An asynchronous method to get the next event that occurs.
    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_stream.next().await
    }

    /// Sends a message to the specified peer.
    ///
    /// A [Event::MessageSent] or
    pub async fn send_message(&self, peer: SocketAddr, message: Vec<u8>) {
        self.send_command(Command::SendMessage {
            message_id: 0,
            addr: peer,
            data: message,
        })
        .await;
    }

    /// Disconnects the specified peer.
    ///
    /// Once fully disconnected, an [Event::ConnectionDisconnected] event will be emitted.
    pub async fn disconnect(&self, peer: SocketAddr) {
        self.send_command(Command::Disconnect { addr: peer }).await;
    }

    /// Attempts to connect to the specified peer.
    ///
    /// A [Event::ConnectionEstablished] or [Event::ConnectionRejected] event will be emitted depending on the result
    /// of the connection attempt.
    pub async fn connect(&self, addr: SocketAddr) {
        self.send_command(Command::Connect { addr }).await;
    }

    /// Shuts down the AMS instance, closing all connections.
    pub async fn shutdown(self) {
        self.manager.shutdown().await;
    }

    /// Sends a command to the manager task.
    async fn send_command(&self, command: Command) {
        self.manager.send_command(command).await;
    }
}

enum Command {
    Connect {
        addr: SocketAddr,
    },
    Disconnect {
        addr: SocketAddr,
    },
    SendMessage {
        message_id: u64,
        addr: SocketAddr,
        data: Vec<u8>,
    },
}

/// Events emitted by the AMS instance via [Ams::next_event].
pub enum Event {
    /// A new connection is being requested
    ConnectionRequested {
        /// The peer address requesting the connection
        peer: SocketAddr,
        /// A channel to respond to the connection request
        response: tokio::sync::oneshot::Sender<bool>,
    },
    /// A connection requested by a peer has been successfully established.
    ConnectionEstablished {
        /// The socket addr of the established connection
        peer: SocketAddr,
    },
    ConnectionRejected {
        /// The socket addr of the rejected connection
        peer: SocketAddr,
    },
    /// A connection not requested by us has been disconnected.
    ConnectionDisconnected {
        /// The socket addr of the disconnected connection
        peer: SocketAddr,
    },
    /// A message received from a peer
    MessageReceived {
        /// The peer address that sent the message
        peer: SocketAddr,
        /// The unique id of the message
        message_id: u64,
        /// The message payload
        payload: Vec<u8>,
        /// The timestamp the message was received
        timestamp: SystemTime,
    },
    /// A message was successfully sent to a peer
    MessageSent {
        /// The peer address the message was sent to
        peer: SocketAddr,
        /// The unique id of the message
        message_id: u64,
        /// The timestamp the message was sent
        timestamp: SystemTime,
    },
    /// A message failed to send to a peer
    MessageFailed {
        /// The peer address the message failed to send to
        peer: SocketAddr,
        /// The unique id of the message
        message_id: u64,
    },
}
