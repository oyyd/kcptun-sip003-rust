use crate::config::Config;
use anyhow::Result;
use log;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream},
};
use tokio_kcp::KcpStream;
use tokio_smux::{Session, Stream};

pub struct Client {
  config: Config,
}

async fn proxy(mut socket: TcpStream, mut smux_stream: Stream) -> Result<()> {
  let mut buf: Vec<u8> = vec![0; 65535];

  loop {
    tokio::select! {
      size = socket.read(&mut buf) => {
        let size = size?;
        if size == 0 {
          break
        }
        let data = &buf[0..size];
        smux_stream.send_message(data.to_vec()).await?;
      }
      msg = smux_stream.recv_message() => {
        let msg = msg?;
        if msg.is_none() {
          // receive fin
          break
        }
        let msg = msg.unwrap();
        socket.write_all(&msg).await?;
      }
    }
  }
  Ok(())
}

impl Client {
  pub fn new(config: Config) -> Self {
    Client { config }
  }

  pub async fn run(&self) -> Result<()> {
    // - create kcp client connects to server
    let remote_addr = self.config.plugin.client_remote_addr()?;
    let kcp_stream = KcpStream::connect(&self.config.kcp, remote_addr).await?;

    // - wrap smux
    let mut session = Session::client(kcp_stream, Config::new_smux())?;

    // - bind local tcp server
    let local_addr = self.config.plugin.client_local_addr()?;

    log::info!("tcp bind local_addr {}", local_addr);

    let tcp_listener = TcpListener::bind(local_addr).await?;

    // - receive tcp clients and create kcp clients stream to proxy them
    loop {
      let (socket, _) = tcp_listener.accept().await?;
      let smux_stream = session.open_stream().await;

      // TODO restart if kcp stream broken?
      if smux_stream.is_err() {
        let err = smux_stream.err().unwrap();
        log::warn!("failed to open smux stream, err: {}", err.to_string());
        continue;
      }

      let smux_stream = smux_stream?;

      tokio::spawn(async move {
        let res = proxy(socket, smux_stream).await;
        let _ = res.map_err(|err| log::warn!("tunnel failure, err: {}", err.to_string()));
      });
    }
  }
}
