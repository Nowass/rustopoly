use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Deserialize, Serialize)]
/// Represents messages exchanged between the client and server in the Rustopoly game.
pub enum RustopolyMessage {
    /// A text message containing a string.
    Text(String),
    /// A requesting to end the connection.
    Quit,
}

/// Custom error type for the crate.
#[derive(Error, Debug)]
pub enum LibError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bincode Serialization  error: {0}")]
    SerializationError(#[from] bincode::Error),
}

/// Struct representing command-line arguments for IPv4 address and port.
///
/// This struct is used to parse command-line arguments for IPv4 address and port number using the clap crate.
#[derive(Parser, Debug)]
pub struct Args {
    /// IPv4 address to connect to or listen on.
    #[clap(short, long, default_value = "127.0.0.1")]
    pub ip: Ipv4Addr,

    /// Port number to use for the connection.
    #[clap(short, long, default_value = "11111")]
    pub port: u16,
}

impl RustopolyMessage {
    /// Receives a RustopolyMessage from the TCP Stream.
    pub async fn receive(stream: &mut TcpStream) -> Result<Self, LibError> {
        // Step 1: Receive the size of encoded payload as a fixed 4-byte message
        let mut size_buffer = [0; 4];
        stream.read_exact(&mut size_buffer).await?;
        let payload_size = u32::from_be_bytes(size_buffer);

        // Step 2: Receive the encoded payload
        let mut encoded = vec![0; payload_size as usize];
        stream.read_exact(&mut encoded).await?;

        // Step 3: Decode (deserialize) payload into RustopolyMessage
        let message = bincode::deserialize(&encoded)?;

        Ok(message)
    }

    /// Sends a RustopolyMessage to the TCP Stream.
    pub async fn send(&self, stream: &mut TcpStream) -> Result<(), LibError> {
        // Step 1: Encode (serialize) the RustopolyMessage
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();

        // Step 2: Send the size of the payload as a fixed 4-byte message
        let payload_size = encoded.len() as u32;
        let size_buffer = payload_size.to_be_bytes();
        stream.write_all(&size_buffer).await?;

        // Step 3: Send encoded payload
        stream.write_all(&encoded).await?;
        stream.flush().await?;

        Ok(())
    }
}
