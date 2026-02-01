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

    let forex_client = Client {
        api_key: cfg.data_api_key.clone(),
        fx_ws: client.fx_ws,
        crypto_ws: None,
        equity_ws: None,
    };

    let crypto_client = Client {
        api_key: cfg.data_api_key.clone(),
        fx_ws: None,
        crypto_ws: client.crypto_ws,
        equity_ws: None,
    };

    let equity_client = Client {
        api_key: cfg.data_api_key.clone(),
        fx_ws: None,
        crypto_ws: None,
        equity_ws: client.equity_ws,
    };

    tokio::spawn(async move {
        match outbound_msg_handler(
            forex_client,
            AssetClass::Forex,
            cfg.forex_tickers,
            deserialize_msg,
        )
        .await
        {
            Ok(_) => info!("Forex handler completed"),
            Err(e) => error!("Forex handler crashed: {}", e),
        }
    });

    tokio::spawn(async move {
        match outbound_msg_handler(
            crypto_client,
            AssetClass::Crypto,
            cfg.crypto_tickers,
            deserialize_msg,
        )
        .await
        {
            Ok(_) => info!("Crypto handler completed"),
            Err(e) => error!("Crypto handler crashed: {}", e),
        }
    });

    tokio::spawn(async move {
        match outbound_msg_handler(
            equity_client,
            AssetClass::Equity,
            cfg.equity_tickers,
            deserialize_msg,
        )
        .await
        {
            Ok(_) => info!("Equity handler completed"),
            Err(e) => error!("Equity handler crashed: {}", e),
        }
    });

    info!("Market data workers started");

    tokio::signal::ctrl_c().await?;
    warn!("Shutdown signal received");

    Ok(())
}
