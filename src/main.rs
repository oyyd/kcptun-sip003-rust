mod client;
mod config;
mod plugin;
mod server;

use anyhow::Result;
use env_logger;
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
  let env = env_logger::Env::new().filter_or("RUST_LOG", "info");
  env_logger::init_from_env(env);

  let plugin_opts = plugin::PluginOptions::new();

  if plugin_opts.is_client {
    client().await.unwrap()
  } else {
    server().await.unwrap()
  }
}
