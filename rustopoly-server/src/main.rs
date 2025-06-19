use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info};
use rustopoly_interface::Args;
use rustopoly_server::{create_server, server_loop};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();
    info!("{:?}", args);

    info!("Creating server socket");
    let server = create_server(args.ip, args.port)
        .await
        .context("Failed to create server")?;
    info!("Starting server loop");
    server_loop(&server).await.context("Server loop crashed")?;
    error!("Server execution finished");
    Ok(())
}
