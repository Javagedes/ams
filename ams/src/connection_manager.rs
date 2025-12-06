use std::{collections::HashMap, time::SystemTime};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

use crate::{Command, api::Message, connection::Connection, layers::transmit};

type Unsecure = (transmit::Transmit,);

// The AMS connection manager, responsible for managing all incoming and active connections to remote peers.
pub(crate) struct ConnectionManager {
    /// A channel to send commands to the manager task.
    sender: mpsc::Sender<Command>,
    /// A token to signal to the manager task to shutdown.
    token: tokio_util::sync::CancellationToken,
    /// The running manager task's join handle.
    handle: tokio::task::JoinHandle<()>,
}

impl ConnectionManager {
    /// Queues a shutdown of the manager and all connections.
    pub(crate) async fn shutdown(self) {
        self.token.cancel();
        let _ = self.handle.await;
    }

    pub(crate) async fn send_command(&self, command: Command) {
        let _ = self.sender.send(command).await;
    }

    /// Spawns a task to manage all incoming and active connections.
    ///
    /// The [Command] enum is used to interact with the manager and its connections.
    pub(crate) async fn spawn(
        addr: impl ToString,
        event_tx: mpsc::UnboundedSender<crate::Event>,
    ) -> std::io::Result<Self> {
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
            let my_addr = listener.local_addr().unwrap();

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    // Handle a new connection
                    Ok((stream, addr)) = listener.accept() => {
                        let (rx, tx) = oneshot::channel();
                        if event_tx.send(crate::Event::ConnectionRequested { peer: addr, response: rx }).is_err() {
                            continue;
                        }
                        if let Ok(true) = tx.await {
                            let conn = Connection::spawn::<Unsecure>(stream, addr, exit_tx.clone());
                            connections.insert(addr, conn);
                            let _ = event_tx.send(crate::Event::ConnectionEstablished { peer: addr });
                        }
                    }
                    // Handle a manager command
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            Command::Disconnect { addr } => {
                                println!("Disconnecting from {addr}");
                                if let Some(connection) = connections.remove(&addr) {
                                    connection.disconnect().await;
                                }
                                event_tx.send(crate::Event::ConnectionDisconnected { peer: addr }).ok();
                            }
                            Command::Connect { addr } => {
                                if let Ok(stream) = TcpStream::connect(&addr).await {
                                    let conn = Connection::spawn::<Unsecure>(stream, addr, exit_tx.clone());
                                    connections.insert(addr, conn);
                                    let _ = event_tx.send(crate::Event::ConnectionEstablished { peer: addr });
                                }
                            }
                            Command::SendMessage { message_id, addr, data } => {
                                let message = Message {
                                    id: message_id,
                                    payload: data,
                                    sender: my_addr.to_string(),
                                };
                                if let Some(conn) = connections.get(&addr) {
                                    conn.send_command(Box::new(crate::layers::transmit::Cmd::SendMessage(message))).await;
                                    let _ = event_tx.send(crate::Event::MessageSent { peer: addr, message_id, timestamp: SystemTime::now() });
                                }
                                else {
                                    let _ = event_tx.send(crate::Event::MessageFailed { peer: addr, message_id });
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
