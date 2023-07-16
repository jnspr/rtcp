mod bridge;
mod proxy_client;
mod proxy_server;

use proxy_client::ProxyClient;
use proxy_server::ProxyServer;
use tokio::io::Result;

enum Mode {
    Server {
        server_endpoint: String,
        client_endpoint: String,
    },
    Client {
        server_endpoint: String,
        local_endpoint: String,
    },
}

impl Mode {
    pub fn parse() -> Option<Mode> {
        let mut iter = std::env::args().skip(1);
        match iter.next()?.as_str() {
            "server" => Some(Mode::Server {
                server_endpoint: iter.next()?,
                client_endpoint: iter.next()?,
            }),
            "client" => Some(Mode::Client {
                server_endpoint: iter.next()?,
                local_endpoint: iter.next()?,
            }),
            _ => None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mode = match Mode::parse() {
        Some(mode) => mode,
        _ => {
            eprintln!("usage: rtcp <mode> ...");
            eprintln!("            server <server-endpoint> <client-endpoint>");
            eprintln!("            client <server-endpoint> <local-endpoint>");
            return Ok(());
        }
    };

    env_logger::init();
    match mode {
        Mode::Server {
            server_endpoint,
            client_endpoint,
        } => {
            ProxyServer::new(&server_endpoint, &client_endpoint)
                .await?
                .serve()
                .await?;
        }
        Mode::Client {
            server_endpoint,
            local_endpoint,
        } => {
            ProxyClient::new(&server_endpoint, &local_endpoint)
                .await?
                .serve()
                .await?;
        }
    }
    Ok(())
}
