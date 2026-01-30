use anyhow::Result;
use mdworkers::services::streams::get_stream_entities;
use mdworkers::types::assetclass::AssetClass;
use mdworkers::types::message::deserialize_msg;
use mdworkers::ws::handlers::outbound_msg_handler;
use mdworkers::{config::cfg::Config, types::client::Client};
use tracing::{error, info, warn};

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load_env()?;

    info!("Initializing RabbitMQ streams...");
    let _stream_entities = get_stream_entities().await?;

    info!("Connecting to WebSocket feeds...");
    let client = Client::new(&cfg).await?;

    let (forex_result, crypto_result, equity_result) = tokio::join!(
        outbound_msg_handler(
            Client {
                api_key: cfg.data_api_key.clone(),
                fx_ws: client.fx_ws,
                crypto_ws: None,
                equity_ws: None,
            },
            AssetClass::Forex,
            deserialize_msg
        ),
        outbound_msg_handler(
            Client {
                api_key: cfg.data_api_key.clone(),
                fx_ws: None,
                crypto_ws: client.crypto_ws,
                equity_ws: None,
            },
            AssetClass::Crypto,
            deserialize_msg
        ),
        outbound_msg_handler(
            Client {
                api_key: cfg.data_api_key.clone(),
                fx_ws: None,
                crypto_ws: None,
                equity_ws: client.equity_ws,
            },
            AssetClass::Equity,
            deserialize_msg
        )
    );

    // Check results
    if let Err(e) = forex_result {
        error!("Forex handler error: {}", e);
    }
    if let Err(e) = crypto_result {
        error!("Crypto handler error: {}", e);
    }
    if let Err(e) = equity_result {
        error!("Equity handler error: {}", e);
    }

    Ok(())
}
