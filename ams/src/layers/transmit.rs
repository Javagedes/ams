//! A controller layer for transmitting and receiving raw messages.
use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{Command, api::Message};

/// A simple Controller layer for transmitting and receiving raw messages.
pub struct Transmit;

impl super::Layer for Transmit {
    type Command = Cmd;

    async fn initialize(_stream: &mut Framed<TcpStream, LengthDelimitedCodec>) -> Self {
        Self
    }

    fn handle_cmd(&mut self, command: Self::Command) -> Option<BytesMut> {
        match command {
            Cmd::SendMessage(message) => {
                let bytes = BytesMut::new();
                let bytes = postcard::to_extend(&message, bytes).unwrap();
                Some(bytes)
            }
        }
    }

    fn handle_outgoing_frame(&mut self, _frame: &mut bytes::BytesMut) {}

    fn handle_incoming_frame(&mut self, frame: &mut bytes::BytesMut) -> Option<Command> {
        if let Ok(msg) = postcard::from_bytes::<Message>(frame) {
            println!(
                "Received message: {}",
                String::from_utf8_lossy(&msg.payload)
            );
            // TODO
        };
        None
    }
}

pub enum Cmd {
    SendMessage(Message),
}
