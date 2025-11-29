#![doc = include_str!("../../README.md")]

pub mod api;
mod connection;
mod controller;
mod layers;

use std::collections::HashMap;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

use crate::{
    api::{ConnectionId, Message},
    connection::Connection,
    layers::transmit,
};

type Unsecure = (transmit::Transmit,);

pub struct Ams {
    /// The connection manager.
    manager: Manager,
}

impl Ams {
    pub async fn bind(addr: impl ToString) -> std::io::Result<Self> {
        Ok(Self {
            manager: Manager::spawn(addr).await?,
        })
    }

    pub async fn shutdown(self) {
        let _ = self.manager.shutdown().await;
    }

    pub async fn connect(&self, addr: impl ToString) -> Option<ConnectionId> {
        let (tx, rx) = oneshot::channel();
        self.manager
            .sender
            .send(Command::Connect(addr.to_string(), tx))
            .await
            .ok();
        rx.await.ok().flatten()
    }

    pub async fn send_message(&self, message: Message) {
        self.manager
            .sender
            .send(Command::SendMessage(message))
            .await
            .ok();
    }
}

/// The AMS connection manager, responsible for managing all incoming and active connections to remote peers.
struct Manager {
    /// A channel to send commands to the manager task.
    sender: mpsc::Sender<Command>,
    /// A token to signal to the manager task to shutdown.
    token: tokio_util::sync::CancellationToken,
    /// The running manager task's join handle.
    handle: tokio::task::JoinHandle<()>,
}

impl Manager {
    /// Queues a shutdown of the manager and all connections.
    async fn shutdown(self) {
        self.token.cancel();
        let _ = self.handle.await;
    }

    /// Spawns a task to manage all incoming and active connections.
    ///
    /// The [Command] enum is used to interact with the manager and its connections.
    pub async fn spawn(addr: impl ToString) -> std::io::Result<Self> {
        // Channel to receive commands for the manager.
        let (tx, mut rx) = mpsc::channel(100);
        let token = tokio_util::sync::CancellationToken::new();
        let cancellation_token = token.clone();

        // Give a copy of the command sender to each connection. This allows them to send commands back to the manager.
        // Namely, to notify it when they are shutting down, so the manager can clean up its state.
        let exit_tx = tx.clone();

        let listener = TcpListener::bind(addr.to_string()).await?;

        let handle = tokio::spawn(async move {
            let mut connections = HashMap::new();

            // TODO: improve this
            let mut next_id = 0usize;
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    // Handle a new connection
                    Ok((stream, _)) = listener.accept() => {
                        let id = ConnectionId { id: next_id, generation: 0 };
                        next_id +=1;
                        let conn = Connection::spawn::<Unsecure>(stream, id, exit_tx.clone());
                        connections.insert(id, conn);

                    }
                    // Handle a manager command
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            Command::Disconnect(id) => {
                                if let Some(connection) = connections.remove(&id) {
                                    connection.disconnect().await;
                                }
                            }
                            Command::Connect(addr, resp_tx) => {
                                if let Ok(stream) = TcpStream::connect(addr.to_string()).await {
                                    let id = ConnectionId { id: next_id, generation: 0 };
                                    next_id +=1;
                                    let conn = Connection::spawn::<Unsecure>(stream, id, exit_tx.clone());
                                    connections.insert(id, conn);
                                    let _ = resp_tx.send(Some(id));
                                }
                                else {
                                    let _ = resp_tx.send(None);
                                }
                            }
                            Command::HandleMessage(_message) => {
                                println!("Received message: {}", _message.payload);
                            }
                            Command::SendMessage(message) => {
                                if let Some(conn) = connections.get(&message.receiver) {
                                    conn.send_command(Box::new(crate::layers::transmit::Cmd::SendMessage(message))).await;
                                }
                            }
                        }
                    }
                }
            }

            futures::future::join_all(connections.into_values().map(|conn| conn.disconnect()))
                .await;
        });

        Ok(Self {
            sender: tx,
            token,
            handle,
        })
    }
}

/// Commands that can be managed directly by the AMS manager.
enum Command {
    /// Disconnect the specified connection.
    Disconnect(ConnectionId),
    /// Send a message to the specified connection.
    SendMessage(Message),
    /// Handle an incoming message from a connection.
    HandleMessage(Message),
    /// Connect to a new address.
    Connect(String, oneshot::Sender<Option<ConnectionId>>),
}
