use clap::{Parser, Subcommand};
use anyhow::Result;
use log::{info, error};

mod config;
mod near_client;
mod node_daemon;
mod ai_engine;
mod task_processor;
mod heartbeat;

use config::NodeConfig;
use node_daemon::NodeDaemon;

#[derive(Parser)]
#[command(name = "deai-node")]
#[command(about = "DeAI Node Client for distributed AI computation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register node with the DeAI network
    Register {
        /// Node configuration file path
        #[arg(short, long, default_value = "node_config.toml")]
        config: String,
    },
    /// Start the node daemon
    Start {
        /// Node configuration file path
        #[arg(short, long, default_value = "node_config.toml")]
        config: String,
    },
    /// Check node status
    Status {
        /// Node configuration file path
        #[arg(short, long, default_value = "node_config.toml")]
        config: String,
    },
    /// Deactivate and withdraw stake
    Deactivate {
        /// Node configuration file path
        #[arg(short, long, default_value = "node_config.toml")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Register { config } => {
            info!("Registering node with config: {}", config);
            let node_config = NodeConfig::load(&config)?;
            let daemon = NodeDaemon::new(node_config).await?;
            daemon.register().await?;
            info!("Node registered successfully!");
        }
        Commands::Start { config } => {
            info!("Starting node daemon with config: {}", config);
            let node_config = NodeConfig::load(&config)?;
            let daemon = NodeDaemon::new(node_config).await?;
            daemon.start().await?;
        }
        Commands::Status { config } => {
            info!("Checking node status with config: {}", config);
            let node_config = NodeConfig::load(&config)?;
            let daemon = NodeDaemon::new(node_config).await?;
            daemon.status().await?;
        }
        Commands::Deactivate { config } => {
            info!("Deactivating node with config: {}", config);
            let node_config = NodeConfig::load(&config)?;
            let daemon = NodeDaemon::new(node_config).await?;
            daemon.deactivate().await?;
        }
    }
    
    Ok(())
}