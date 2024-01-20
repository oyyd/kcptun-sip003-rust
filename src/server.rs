use std::net::SocketAddr;

use crate::config::Config;
use anyhow::Result;
use log;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpSocket, TcpStream},
};
use tokio_kcp::KcpListener;
use tokio_smux::{Session, Stream};

pub struct Server {
  config: Config,
}

const MAX_FRAME_DATA_LEN: usize = 2048;

async fn proxy(mut socket: TcpStream, mut smux_stream: Stream) -> Result<()> {
  let mut buf: Vec<u8> = vec![0; 65535];

  loop {
    tokio::select! {
      size = socket.read(&mut buf) => {
        let size = size?;
        if size == 0 {
          break;
        }

        // NOTE: When test, large frames will crash shadowrocket clients.
        // But I can't reproduce it with kcptun.
        let mut pos = 0;
        loop {
          if pos >= size {
            break;
          }
          let mut end = pos + MAX_FRAME_DATA_LEN;
          if end > size {
            end = size;
          }
          let data = &buf[pos..end];
          smux_stream.send_message(data.to_vec()).await?;
          pos = end;
        };

        // NOTE: Simply write all of them into a frame when using with kcptun.
        // smux_stream.send_message(buf[0..size].to_vec()).await?;
      }
      msg = smux_stream.recv_message() => {
        let msg = msg?;
        if msg.is_none() {
          break;
        }

        let data = msg.unwrap();
        socket.write_all(&data).await?;
      }
    }
  }

  Ok(())
}

async fn handle_stream(
  smux_stream: Stream,
  local_addr: SocketAddr,
  target_addr: SocketAddr,
  sockbuf: u32,
) -> Result<()> {
  let socket = {
    match target_addr.is_ipv4() {
      true => TcpSocket::new_v4(),
      false => TcpSocket::new_v6(),
    }
  }?;

  socket.set_recv_buffer_size(sockbuf)?;
  socket.set_send_buffer_size(sockbuf)?;

  let tcp_socket = socket.connect(target_addr).await?;

  log::info!("proxy {} -> {}", local_addr, target_addr);

  proxy(tcp_socket, smux_stream).await?;

  Ok(())
}

async fn loop_smux_session(
  mut session: Session,
  local_addr: SocketAddr,
  target_addr: SocketAddr,
  sockbuf: u32,
) -> Result<()> {
  // - accept smux stream
  loop {
    let smux_stream = session.accept_stream().await?;
    let sid = smux_stream.sid();

    log::trace!("accept smux stream, sid {}", sid);

    tokio::spawn(async move {
      // - create tcp socket connect to the remote and proxy their data
      let res = handle_stream(smux_stream, local_addr, target_addr.clone(), sockbuf).await;
      let _ = res.map_err(|err| {
        log::warn!("proxy failed, err {}", err.to_string());
      });
      log::trace!("smux stream finished, sid {}", sid);
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

    log::info!(
      "server starts, listen_addr={}, target_addr={}",
      listen_addr,
      target_addr
    );

    let mut listener = KcpListener::bind(self.config.kcp, listen_addr).await?;
    let sockbuf = self.config.sockbuf;

    // - accept kcp client stream
    loop {
      let (kcp_stream, addr) = listener.accept().await?;

      log::info!("accept kcp stream, addr {}", addr);

      // - wrap smux
      let session = Session::server(kcp_stream, Config::new_smux())?;

      tokio::spawn(async move {
        let res = loop_smux_session(session, addr, target_addr, sockbuf).await;
        let _ = res.map_err(|err| {
          log::warn!("smux session failed, err: {}", err.to_string());
        });
        log::info!("kcp stream finished, addr {}", addr);
      });
    }
  }
}
