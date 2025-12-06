use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use std::any::Any;

use crate::layers::Layer;

/// A Controller is responsible for processing frames from a remote peer or commands from the AMS manager.
///
/// While this trait could be implemented directly, it is intended to be composed of multiple [Layer]s to form a
/// processing pipeline. Since this is the intended usage, documentation regarding the trait method behaviors
/// will refer to the layered usage.
pub trait Controller: Send + 'static {
    /// Initializes each layer in the controller stack, returning a tuple of all layers initialied state.
    fn initialize(
        stream: &mut Framed<TcpStream, LengthDelimitedCodec>,
    ) -> impl std::future::Future<Output = Self> + std::marker::Send
    where
        Self: Sized + Send;

    /// Processes a command from the manager.
    ///
    /// This method will search through each layer in the controller stack to find the layer that can handle the
    /// command. Once found, it will call that layer's [Layer::handle_cmd] method. If the layer returns some bytes,
    /// those bytes will be sent back up the layer stack from it's current location to be transmitted to the remote
    /// peer.
    fn process_cmd(&mut self, cmd: Box<dyn std::any::Any + Send>) -> Option<BytesMut>;

    /// Process an incoming frame from a remote peer.
    ///
    /// This method will pass the frame through each layer in the controller stack, allowing each layer to inspect and
    /// modify the frame as needed. Any layer may return a [crate::Command], which will be collected and sent back
    /// to the manager after all layers have processed the frame.
    fn process_incoming_frame(&mut self, frame: &mut bytes::BytesMut) -> Vec<crate::Command>;
}

// TODO: Turn this into a proc macro
#[allow(unused_mut)]
#[allow(non_snake_case)]
impl<L1: Layer> Controller for (L1,) {
    async fn initialize(stream: &mut Framed<TcpStream, LengthDelimitedCodec>) -> Self
    where
        Self: Sized + Send,
    {
        (L1::initialize(stream).await,)
    }

    fn process_cmd(&mut self, cmd: Box<dyn Any + Send>) -> Option<BytesMut> {
        let (L1,) = self;

        if cmd.is::<L1::Command>() {
            let mut bytes = L1.handle_cmd(
                *cmd.downcast::<L1::Command>()
                    .expect("type validated through Any::is."),
            );

            return bytes;
        }
        None
    }

    fn process_incoming_frame(&mut self, mut frame: &mut BytesMut) -> Vec<crate::Command> {
        let (L,) = self;
        let mut cmds = Vec::new();

        if let Some(cmd) = L.handle_incoming_frame(frame) {
            cmds.push(cmd);
        }

        cmds
    }
}

#[allow(unused_mut)]
#[allow(non_snake_case)]
impl<L1: Layer, L2: Layer> Controller for (L1, L2) {
    async fn initialize(stream: &mut Framed<TcpStream, LengthDelimitedCodec>) -> Self {
        (L1::initialize(stream).await, L2::initialize(stream).await)
    }

    fn process_cmd(&mut self, cmd: Box<dyn Any + Send>) -> Option<BytesMut> {
        let (L1, L2) = self;

        if cmd.is::<L1::Command>() {
            let mut bytes = L1.handle_cmd(
                *cmd.downcast::<L1::Command>()
                    .expect("type validated through Any::is."),
            );

            return bytes;
        }

        if cmd.is::<L2::Command>() {
            let mut bytes = L2.handle_cmd(
                *cmd.downcast::<L2::Command>()
                    .expect("type validated through Any::is."),
            );

            if let Some(ref mut bytes) = bytes {
                L1.handle_outgoing_frame(bytes);
            }

            return bytes;
        }
        None
    }

    fn process_incoming_frame(&mut self, frame: &mut bytes::BytesMut) -> Vec<crate::Command> {
        let (L1, L2) = self;
        let mut cmds = Vec::new();
        let mut frame_ref = frame;

        if let Some(cmd) = L2.handle_incoming_frame(frame_ref) {
            cmds.push(cmd);
        }

        if let Some(cmd) = L1.handle_incoming_frame(frame_ref) {
            cmds.push(cmd);
        }
        cmds
    }
}

#[allow(unused_mut)]
#[allow(non_snake_case)]
impl<L1: Layer, L2: Layer, L3: Layer> Controller for (L1, L2, L3) {
    async fn initialize(stream: &mut Framed<TcpStream, LengthDelimitedCodec>) -> Self {
        (
            L1::initialize(stream).await,
            L2::initialize(stream).await,
            L3::initialize(stream).await,
        )
    }

    fn process_cmd(&mut self, cmd: Box<dyn Any + Send>) -> Option<BytesMut> {
        let (L1, L2, L3) = self;

        if cmd.is::<L1::Command>() {
            let mut bytes = L1.handle_cmd(
                *cmd.downcast::<L1::Command>()
                    .expect("type validated through Any::is."),
            );

            return bytes;
        }

        if cmd.is::<L2::Command>() {
            let mut bytes = L2.handle_cmd(
                *cmd.downcast::<L2::Command>()
                    .expect("type validated through Any::is."),
            );

            if let Some(ref mut bytes) = bytes {
                L1.handle_outgoing_frame(bytes);
            }

            return bytes;
        }

        if cmd.is::<L3::Command>() {
            let mut bytes = L3.handle_cmd(
                *cmd.downcast::<L3::Command>()
                    .expect("type validated through Any::is."),
            );

            if let Some(ref mut bytes) = bytes {
                L2.handle_outgoing_frame(bytes);
                L1.handle_outgoing_frame(bytes);
            }

            return bytes;
        }
        None
    }

    fn process_incoming_frame(&mut self, frame: &mut bytes::BytesMut) -> Vec<crate::Command> {
        let (L1, L2, L3) = self;
        let mut cmds = Vec::new();
        let mut frame_ref = frame;

        if let Some(cmd) = L3.handle_incoming_frame(frame_ref) {
            cmds.push(cmd);
        }

        if let Some(cmd) = L2.handle_incoming_frame(frame_ref) {
            cmds.push(cmd);
        }

        if let Some(cmd) = L1.handle_incoming_frame(frame_ref) {
            cmds.push(cmd);
        }
        cmds
    }
}
