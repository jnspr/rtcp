use crate::bridge::bridge;
use std::net::SocketAddr;
use tokio::io::{AsyncWriteExt, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;

pub struct ProxyServer {
    server_listener: TcpListener,
    client_listener: TcpListener,
    active_server: Option<(TcpStream, SocketAddr)>,
}

impl ProxyServer {
    pub async fn new(server_endpoint: &str, client_endpoint: &str) -> Result<ProxyServer> {
        Ok(ProxyServer {
            server_listener: TcpListener::bind(server_endpoint).await?,
            client_listener: TcpListener::bind(client_endpoint).await?,
            active_server: None,
        })
    }

    pub async fn serve(mut self) -> Result<()> {
        loop {
            select! {
                Ok(server) = self.server_listener.accept() => {
                    log::info!("A server has been accepted ({})", server.1);
                    self.active_server = Some(server);
                }

                Ok((client, client_address)) = self.client_listener.accept() => {
                    if let Some((mut server, server_address)) = self.active_server.take() {
                        tokio::spawn(async move {
                            if server.write_all(&[0xff_u8]).await.is_err() {
                                log::warn!("Unable to send bridge request to {}", server_address);
                                return;
                            }
                            log::debug!("Established: {} <-> {}", server_address, client_address);
                            drop(bridge(server, client).await);
                            log::debug!("Disconnected: {} <-> {}", server_address, client_address);
                        });
                    } else {
                        log::warn!("A client connected without an active server");
                    }
                }
            }
        }
    }
}
