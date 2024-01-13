mod client;
mod config;
mod plugin;
mod server;

use anyhow::Result;
use tokio;

async fn server() -> Result<()> {
  let config = config::Config::new_server()?;

  let s = server::Server::new(config);

  s.run().await?;

  Ok(())
}

async fn client() -> Result<()> {
  let config = config::Config::new_client()?;

  let c = client::Client::new(config);

  c.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() {
  client().await.unwrap();
}
