use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use rustopoly_client::{client_loop, connect_client};
use rustopoly_interface::Args;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();
    info!("{:?}", args);

    info!("Connecting to server");
    let mut conn = connect_client(args.ip, args.port)
        .await
        .context("Failed to connect to server")?;
    info!("Starting client loop");
    client_loop(&mut conn)
        .await
        .context("Client loop crashed")?;
    info!("Client execution finished");
    Ok(())
}
