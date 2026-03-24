mod agent;
mod host;
mod protocol;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shim", about = "romhack-playground container bridge")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Run as the MCP server on the host
    Host,
    /// Run as the in-container agent (reads typed ops from stdin)
    Agent,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Host => host::run().await,
        Mode::Agent => agent::run().await,
    }
}
