use crate::plugin::PluginConfig;
use anyhow::Result;
use tokio_kcp::{KcpConfig, KcpNoDelayConfig};
use tokio_smux::SmuxConfig;

pub struct Config {
  pub plugin: PluginConfig,
  pub kcp: KcpConfig,
  pub smux: SmuxConfig,
}

impl Config {
  pub fn new_client() -> Result<Self> {
    let plugin = PluginConfig::new_client()?;
    let kcp = Config::new_kcp_config();
    let smux = Config::new_smux();

    Ok(Self { plugin, kcp, smux })
  }

  pub fn new_server() -> Result<Self> {
    let plugin = PluginConfig::new_server()?;
    let kcp = Config::new_kcp_config();
    let smux = Config::new_smux();

    Ok(Self { plugin, kcp, smux })
  }

  fn new_kcp_config() -> KcpConfig {
    let mut kcp = KcpConfig::default();

    // TODO support kcp config from outside
    kcp.nodelay = KcpNoDelayConfig::fastest();

    kcp
  }

  // TODO support config from outside
  pub fn new_smux() -> SmuxConfig {
    let smux = SmuxConfig::default();

    smux
  }
}
