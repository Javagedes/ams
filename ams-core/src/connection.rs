//! A module for managing connections to remote AMS peers.
use std::any::Any;

use futures_util::sink::SinkExt;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{Command, api::ConnectionId, controller::Controller};

/// A connection to a remote AMS peer.
///
/// This struct manages a single connection to a remote AMS peer. During initialization with [Self::spawn], a new task
/// is created to handle the connection's lifecycle. This struct manages a token to signal terminate the connection to
/// the peer and close the task, and a channel to send commands to the underlying controller layer. During
/// initialization, a channel to the base manager is also provided, allowing the connection to schedule commands to be
/// processed against the entire AMS system.
///
/// ## Connection Lifecycle
///
/// 1. Layer negotiaion to establish a common set of layers between the local and remote peer to ensure data frames are
///    properly processed on each side.
/// 2. Layer initialization. This way include additional communication with the remote peer to establish state (e.g. to
///    set up encryption keys).
/// 3. Normal operation where frames or commands are processed through the controller layers.
///
/// ## Layer negotiation
///
/// Layer negotiation is not as dynamic as it might sound. There is a fixed set of Controller implementations  (ordered
/// layers) that are supported by the AMS system. Controllers are first and for-most used for maintainability of the
/// codebase, It helps ensure the decoupling of functionality while also being a point of abstraction for future
/// features The main dynamic aspect of the Controller functionality is to support communicating with the few types of
/// remote peers available (A server, a client with encryption, a client without encryption, etc.). See [Controller]
/// for more information.
pub(crate) struct Connection {
    /// A channel to send commands to the connection's running task.
    sender: mpsc::Sender<Box<dyn Any + Send>>,
    /// A token to signal to the connection's running task to disconnect from the remote peer and shutdown.
    token: tokio_util::sync::CancellationToken,
    /// The running task's join handle so it is possible to await its termination.
    handle: tokio::task::JoinHandle<()>,
}

impl Connection {
    /// Spawns a task to manage the peer connection.
    ///
    /// The task will run until the connection is terminated, either by the remove peer or by calling
    /// [Self::disconnect]. This controller ultimetly wakes up and responds to three different events:
    ///
    /// 1. The cancellation token is triggered, typically by calling [Self::disconnect]. This will result in the
    ///    connection sending a disconnect message to the manager (so the manager can clean up its state) and then self
    ///    terminating.
    /// 2. A command from the manager is received. This command is processed by the underlying controller's
    ///    [Controller::process_cmd] method.
    pub fn spawn<C: Controller>(
        stream: TcpStream,
        id: ConnectionId,
        manager_tx: mpsc::Sender<Command>,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel(32);
        let token = tokio_util::sync::CancellationToken::new();
        let cancellation_token = token.clone();

        let handle = tokio::spawn(async move {
            let framed = Framed::new(stream, LengthDelimitedCodec::new());
            tokio::pin!(framed);

            let mut layers = C::initialize(&mut framed).await;

            loop {
                tokio::select! {
                    // The manager has signaled for this connection to shutdown.
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    // A command from the manager was sent. Process it through the controller layers.
                    Some(cmd) = rx.recv() => {
                        if let Some(bytes) = layers.process_cmd(cmd) {
                            if framed.send(bytes.freeze()).await.is_err() {
                                let _ = manager_tx.send(Command::Disconnect(id)).await;
                            }
                        }
                    }
                    // An incoming frame from the remote peer.
                    maybe_frame = framed.next() => {
                        match maybe_frame {
                            // Successfully received a frame. Process it through the controller layers.
                            Some(Ok(mut frame)) => {
                                for cmd in layers.process_incoming_frame(&mut frame) {
                                    let _ = manager_tx.send(cmd).await;
                                }
                            }
                            // Some error (or disconnect) occured. Notify the manager to clean up state and send a final
                            // disconnect message to this task.
                            Some(Err(_)) | None => {
                                let _ = manager_tx.send(Command::Disconnect(id)).await;
                            }
                        }
                    }
                }
            }
        });

        Self {
            sender: tx,
            token,
            handle,
        }
    }

    /// Sends a command to the underlying connection controller.
    pub async fn send_command(&self, command: Box<dyn Any + Send>) {
        let _ = self.sender.send(command).await;
    }

    /// Gracefully disconnects the connection.
    pub async fn disconnect(self) {
        self.token.cancel();
        let _ = self.handle.await;
    }
}
