use anyhow::{Context, Result};
use log::{info, trace};
use rustopoly_interface::RustopolyMessage;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

/// Creates a TcpStream to connect to the specified IP and port.
pub async fn connect_client(ip: Ipv4Addr, port: u16) -> Result<TcpStream> {
    // Use the provided IP and port to create the socket address
    let server_addr = SocketAddr::V4(SocketAddrV4::new(ip, port));
    trace!("Connecting...");
    let stream = TcpStream::connect(server_addr)
        .await
        .context("Failed to connect to TCP stream")?;
    trace!("Local address: {}", stream.local_addr().unwrap());
    info!("Connected to {server_addr}");
    Ok(stream)
}

/// Main loop to handle communication with the server.
pub async fn client_loop(stream: &mut TcpStream) -> Result<()> {
    // Initialize standard input interface
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    loop {
        // Read user input from stdin
        info!("Insert the request string or 'quit' to terminate connection");
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .await
            .context("Failed to read a line from stdin")?;
        // Trim leading and trailing whitespaces
        let input = input.trim().to_string();

        // Create a Message based on the input command
        let message = if input.starts_with("quit") {
            // Connection quit request
            RustopolyMessage::Quit
        } else {
            RustopolyMessage::Text(input)
        };
        info!("Sending request {:?}", message);
        message
            .send(stream)
            .await
            .context("Requset sending failed")?;
        trace!("Waiting for the response");
        // Receive the response from the server
        let response = RustopolyMessage::receive(stream)
            .await
            .context("Response receiving failed")?;
        info!("Received response: {:?}", response);
        // End the client loop when quit response is received
        if let RustopolyMessage::Quit = response {
            info!("Quit request confirmed");
            return Ok(());
        }

        // Sleep for 1s
        sleep(Duration::from_millis(1000)).await;
    }
}
