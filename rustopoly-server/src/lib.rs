use anyhow::{Context, Result};
use log::{error, info, trace};
use rustopoly_interface::RustopolyMessage;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod financial_mgmt;
mod memory_mgmt;

/// Creates a TcpListener bound to the specified IP and port.
pub async fn create_server(ip: Ipv4Addr, port: u16) -> Result<TcpListener> {
    // Use the provided IP and port
    let sock_addr = SocketAddr::V4(SocketAddrV4::new(ip, port));
    trace!("Binding...");
    // Bind the TcpListener to the socket address
    let listener = TcpListener::bind(sock_addr)
        .await
        .context("Failed to bind TCP listener")?;
    info!("Listener binded to {sock_addr}");
    Ok(listener)
}

/// Main loop to accept and handle incoming client connections.
pub async fn server_loop(listener: &TcpListener) -> Result<()> {
    loop {
        match listener.accept().await {
            Ok((mut stream, peer_addr)) => {
                info!("Accepted connection from {:?}", peer_addr);
                // Spawn a new task to handle each client connection
                match handle_client(&mut stream).await {
                    Ok(_) => info!("Client {:?} handled successfully", peer_addr),
                    Err(e) => error!("Error handling client {:?}: {}", peer_addr, e),
                }
            }
            Err(_e) => {
                error!("Failed to accept connection")
            }
        }
    }
}

/// Handles communication with a single client.
async fn handle_client(stream: &mut TcpStream) -> Result<()> {
    // Get the client's address
    let peer = stream
        .peer_addr()
        .context("Failed to obtain client's address")?;
    loop {
        // Receive a request from the client
        let request = RustopolyMessage::receive(stream)
            .await
            .context("Failed to receive request")?;
        trace!("Received '{:?}' from '{}'", request, peer);

        match request {
            RustopolyMessage::Quit => {
                let response = RustopolyMessage::Quit;
                trace!("Sending response '{response:?}' to '{peer}'");
                // Send the response to the client
                response
                    .send(stream)
                    .await
                    .context("Failed to send the response")?;
                info!("Shutting down connection with {peer}");
                stream
                    .shutdown()
                    .await
                    .context("Failed to terminate connection")?;
                // End the client handling
                return Ok(());
            }
            RustopolyMessage::Text(request_string) => {
                // Echo the request string
                let response = RustopolyMessage::Text(format!("Echo server: '{request_string}'"));
                trace!("Sending response '{response:?}' to '{peer}'");
                // Send the response to the client
                response
                    .send(stream)
                    .await
                    .context("Failed to send the response")?;
            }
        }
    }
}
