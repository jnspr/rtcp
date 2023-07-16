use crate::bridge::bridge;
use std::time::Duration;
use tokio::io::{AsyncReadExt, Result};
use tokio::net::TcpStream;

pub struct ProxyClient {
    server_endpoint: String,
    local_endpoint: String,
}

impl ProxyClient {
    pub async fn new(server_endpoint: &str, local_endpoint: &str) -> Result<ProxyClient> {
        Ok(ProxyClient {
            server_endpoint: server_endpoint.to_owned(),
            local_endpoint: local_endpoint.to_owned(),
        })
    }

    pub async fn serve(mut self) -> Result<()> {
        'retry: loop {
            log::info!("Connecting to proxy server");
            let mut server = match TcpStream::connect(&self.server_endpoint).await {
                Ok(server) => server,
                _ => {
                    log::warn!("Unable to connect to proxy server, retrying in 5 seconds");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue 'retry;
                }
            };

            log::info!("Waiting for proxy request");
            let mut buffer = [0_u8];
            if server.read_exact(&mut buffer).await.is_err() {
                log::warn!("Proxy server closed connection, reconnecting");
                continue 'retry;
            }
            if buffer[0] != 0xff {
                log::warn!("Proxy server sent invalid request, reconnecting");
                continue 'retry;
            }

            log::info!("Connecting to local server");
            let local = match TcpStream::connect(&self.local_endpoint).await {
                Ok(local) => local,
                _ => {
                    log::warn!("Unable to connect to local server, restarting");
                    continue 'retry;
                }
            };

            tokio::spawn(bridge(server, local));
        }
    }
}
