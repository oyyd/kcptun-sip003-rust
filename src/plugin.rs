use anyhow::Result;
use std::env;
use std::net;
use std::str::FromStr;

const LOCAL_HOST: &str = "SS_LOCAL_HOST";
const LOCAL_PORT: &str = "SS_LOCAL_PORT";
const REMOTE_HOST: &str = "SS_REMOTE_HOST";
const REMOTE_PORT: &str = "SS_REMOTE_PORT";

pub struct PluginConfig {
  is_client: bool,

  local_host: String,
  local_port: u16,
  remote_host: String,
  remote_port: u16,
}

impl Default for PluginConfig {
  fn default() -> Self {
    Self {
      is_client: false,
      local_host: "127.0.0.1".to_string(),
      local_port: 12948,
      remote_host: "127.0.0.1".to_string(),
      remote_port: 29900,
    }
  }
}

fn parse_port(val: &str) -> Result<u16> {
  let port = val.parse::<u16>()?;

  Ok(port)
}

impl PluginConfig {
  pub fn new_client() -> Result<Self> {
    let mut config = PluginConfig::new()?;

    config.is_client = true;

    Ok(config)
  }

  pub fn new_server() -> Result<Self> {
    let mut config = PluginConfig::new()?;

    config.is_client = false;

    Ok(config)
  }

  fn new() -> Result<Self> {
    let mut config = Self::default();

    let vars = env::vars();

    for (key, value) in vars {
      match key.as_str() {
        LOCAL_HOST => {
          config.local_host = value;
        }
        LOCAL_PORT => {
          config.local_port = parse_port(&value)?;
        }
        REMOTE_HOST => {
          config.remote_host = value;
        }
        REMOTE_PORT => {
          config.remote_port = parse_port(&value)?;
        }
        _ => {}
      }
    }

    Ok(config)
  }

  pub fn server_listen_addr(&self) -> Result<net::SocketAddr> {
    let addr = format!("{}{}", self.remote_host, self.remote_port);
    let addr = net::SocketAddr::from_str(&addr)?;
    Ok(addr)
  }

  pub fn server_target_addr(&self) -> Result<net::SocketAddr> {
    let addr = format!("{}{}", self.local_host, self.local_port);
    let addr = net::SocketAddr::from_str(&addr)?;
    Ok(addr)
  }

  // TODO check
  pub fn client_local_addr(&self) -> Result<net::SocketAddr> {
    let addr = format!("{}{}", self.local_host, self.local_port);
    let addr = net::SocketAddr::from_str(&addr)?;
    Ok(addr)
  }

  pub fn client_remote_addr(&self) -> Result<net::SocketAddr> {
    let addr = format!("{}{}", self.remote_host, self.remote_port);
    let addr = net::SocketAddr::from_str(&addr)?;
    Ok(addr)
  }
}
