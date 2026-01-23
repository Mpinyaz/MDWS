use crate::{config::cfg::Config, types::assetclass::AssetClass};
use anyhow::Result;
use std::error::Error;
use std::fmt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Client {
    pub fx_ws: Option<WsStream>,
    pub crypto_ws: Option<WsStream>,
    pub equity_ws: Option<WsStream>,
}

pub enum ClientError {
    InternalServerError,
    ConnectionFailed(String),
    InvalidResponse(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InternalServerError => write!(f, "Internal server error occurred"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for ClientError {}

impl Client {
    async fn connect(cfg: &Config, ac: AssetClass) -> Result<WsStream> {
        let url = format!("{}/{}", cfg.url, ac);
        info!("Connecting to WebSocket at: {}", url);

        let (ws_stream, response) = connect_async(&url).await?;

        info!(
            "WebSocket connection established for {:?}. Response: {:?}",
            ac,
            response.status()
        );

        Ok(ws_stream)
    }

    pub async fn new(cfg: &Config) -> Result<Self> {
        info!("Initializing Client with all WebSocket connections");

        let fx_ws = match Self::connect(cfg, AssetClass::Forex).await {
            Ok(ws) => Some(ws),
            Err(e) => {
                error!("Failed to connect to Forex WebSocket: {}", e);
                None
            }
        };

        let crypto_ws = match Self::connect(cfg, AssetClass::Crypto).await {
            Ok(ws) => Some(ws),
            Err(e) => {
                error!("Failed to connect to Crypto WebSocket: {}", e);
                None
            }
        };

        let equity_ws = match Self::connect(cfg, AssetClass::Equity).await {
            Ok(ws) => Some(ws),
            Err(e) => {
                error!("Failed to connect to Equity WebSocket: {}", e);
                None
            }
        };

        Ok(Client {
            fx_ws,
            crypto_ws,
            equity_ws,
        })
    }
}
