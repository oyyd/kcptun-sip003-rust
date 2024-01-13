use std::net::SocketAddr;

use crate::config::Config;
use anyhow::Result;
use log;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream},
};
use tokio_kcp::{KcpListener, KcpStream};
use tokio_smux::{Session, Stream};

pub struct Server {
  config: Config,
}

async fn proxy(mut socket: TcpStream, mut smux_stream: Stream) -> Result<()> {
  let mut buf: Vec<u8> = vec![0; 65535];

  loop {
    tokio::select! {
      size = socket.read(&mut buf) => {
        let size = size?;
        if size == 0 {
          break;
        }

        smux_stream.send_message(buf[0..size].to_vec()).await?;
      }
      msg = smux_stream.recv_message() => {
        let msg = msg?;
        if msg.is_none() {
          break;
        }

        socket.write_all(&msg.unwrap()).await?;
      }
    }
  }

  Ok(())
}

async fn loop_smux_session(mut session: Session, target_addr: SocketAddr) -> Result<()> {
  // - accept smux stream
  loop {
    let smux_stream = session.accept_stream().await?;

    // - create tcp socket connect to the remote and proxy their data
    let tcp_socket = TcpStream::connect(target_addr).await?;

    tokio::spawn(async move {
      let res = proxy(tcp_socket, smux_stream).await;
      let _ = res.map_err(|err| {
        log::warn!("proxy failed, err {}", err.to_string());
      });
    });
  }
}

impl Server {
  pub fn new(config: Config) -> Self {
    Self { config }
  }

  pub async fn run(&self) -> Result<()> {
    // - create kcp stream server
    let listen_addr = self.config.plugin.server_listen_addr()?;
    let target_addr = self.config.plugin.server_target_addr()?;

    let mut listener = KcpListener::bind(self.config.kcp, listen_addr).await?;

    // - accept kcp client stream
    loop {
      let (kcp_stream, _) = listener.accept().await?;

      // - wrap smux
      let session = Session::server(kcp_stream, Config::new_smux())?;

      tokio::spawn(async move {
        let res = loop_smux_session(session, target_addr).await;
        let _ = res.map_err(|err| {
          log::warn!("smux session failed, err: {}", err.to_string());
        });
      });
    }
  }
}
