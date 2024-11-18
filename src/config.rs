use core::time;

use crate::plugin::PluginConfig;
use anyhow::Result;
use tokio_kcp::{KcpConfig, KcpNoDelayConfig};
use tokio_smux::SmuxConfig;

#[derive(Clone)]
pub struct Config {
  pub plugin: PluginConfig,
  pub kcp: KcpConfig,
  pub sockbuf: u32,
  pub server_kcp_stream_read_timeout: Option<time::Duration>,
}

impl Config {
  fn default_sokcbuf() -> u32 {
    1024 * 1024 * 4
  }

  pub fn new_client() -> Result<Self> {
    let plugin = PluginConfig::new_client()?;
    let kcp = Config::new_kcp_config();

    Ok(Self {
      plugin,
      kcp,
      sockbuf: Self::default_sokcbuf(),
      server_kcp_stream_read_timeout: None,
    })
  }

  pub fn new_server() -> Result<Self> {
    let plugin = PluginConfig::new_server()?;
    let kcp = Config::new_kcp_config();

    Ok(Self {
      plugin,
      kcp,
      sockbuf: Self::default_sokcbuf(),
      server_kcp_stream_read_timeout: Some(time::Duration::from_secs(5)),
    })
  }

  fn new_kcp_config() -> KcpConfig {
    let mut kcp = KcpConfig::default();

    // TODO support kcp config from outside
    kcp.nodelay = KcpNoDelayConfig::fastest();

    kcp
  }

  // TODO support config from outside
  // TODO allow clone SmuxConfig and move to config
  pub fn new_smux() -> SmuxConfig {
    let smux = SmuxConfig::default();

    smux
  }
}
