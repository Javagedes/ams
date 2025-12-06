pub mod transmit;

use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub trait Layer: Send + 'static {
    type Command: Send + 'static;

    /// Initializes the layer.
    fn initialize(
        stream: &mut Framed<TcpStream, LengthDelimitedCodec>,
    ) -> impl std::future::Future<Output = Self> + std::marker::Send;

    /// handles a command sent to this layer.
    fn handle_cmd(&mut self, command: Self::Command) -> Option<BytesMut>;

    /// Manipulates an incoming frame sent from the remote peer.
    ///
    /// Returns a ManagerCmd if the frame results in an action required by the AMS manager.
    fn handle_incoming_frame(&mut self, frame: &mut bytes::BytesMut) -> Option<crate::Command>;

    /// Manipulates an outgoing frame before it is sent to the remote peer.
    fn handle_outgoing_frame(&mut self, frame: &mut bytes::BytesMut);
}
