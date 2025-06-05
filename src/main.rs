mod config;
mod listing;
mod random_file_server;

use anyhow::Result;
use random_file_server::RandomFileServer;

fn main() -> Result<()> {
    RandomFileServer::new().start()?;
    Ok(())
}
