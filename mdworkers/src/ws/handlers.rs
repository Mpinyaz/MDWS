use crate::{
    services::streams::get_stream_entities,
    types::{
        assetclass::AssetClass,
        client::{Client, WsStream},
        message::{MsgError, SubscribeData, SubscribeRequest, WsResponse},
    },
};
use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use rabbitmq_stream_client::types::Message as StreamMessage;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};

pub async fn outbound_msg_handler<F>(
    mut client: Client,
    asset_class: AssetClass,
    tickers: Vec<String>,
    mut on_message: F,
) -> Result<(), MsgError>
where
    F: FnMut(Message) -> Result<WsResponse, MsgError>,
{
    let mut conn = match asset_class {
        AssetClass::Forex => client.fx_ws.take(),
        AssetClass::Crypto => client.crypto_ws.take(),
        AssetClass::Equity => client.equity_ws.take(),
    }
    .ok_or_else(|| MsgError::ReadError(format!("{:?} not connected", asset_class)))?;

    let init_seed = SubscribeData {
        subscription_id: None,
        threshold_level: Some("5".to_string()),
        tickers: Some(tickers),
    };

    subscribe_data(client.api_key, &mut conn, init_seed).await?;
    info!("Subscribed to {:?}", asset_class);

    let (_, mut read) = conn.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                match &message {
                    Message::Text(_) | Message::Binary(_) => {
                        match on_message(message) {
                            Ok(ws_response) => {
                                info!(
                                    "Received {:?} message type: {} payload: {:?}",
                                    asset_class, ws_response.message_type, ws_response.data
                                );
                                if let Err(e) = publish_data(ws_response).await {
                                    error!("Publish failed for {:?}: {}", asset_class, e);
                                }
                            }
                            Err(e) => {
                                error!("Error parsing message for {:?}: {}", asset_class, e);
                                // Continue - don't crash
                            }
                        }
                    }
                    Message::Ping(_) => {
                        info!("Received ping for {:?}", asset_class);
                    }
                    Message::Pong(_) => {
                        info!("Received pong for {:?}", asset_class);
                    }
                    Message::Close(frame) => {
                        info!("Connection closed for {:?}: {:?}", asset_class, frame);
                        break; // Exit gracefully
                    }
                    Message::Frame(_) => {
                        // Raw frame, ignore
                    }
                }
            }
            Err(e) => {
                error!("WebSocket error for {:?}: {}", asset_class, e);
                return Err(MsgError::ReadError(format!("WebSocket error: {}", e)));
            }
        }
    }

    info!("{:?} handler completed normally", asset_class);
    Ok(())
}

pub async fn subscribe_data(
    api_key: String,
    conn: &mut WsStream,
    event_data: SubscribeData,
) -> Result<(), MsgError> {
    let subscribe_req = SubscribeRequest {
        event_name: "subscribe".to_string(),
        authorization: api_key,
        event_data,
    };
    let payload = serde_json::to_string(&subscribe_req)
        .map_err(|e| MsgError::SendError(format!("Serialization error: {}", e)))?;
    info!("Subscription payload: {}", payload);
    conn.send(Message::Text(payload.into()))
        .await
        .map_err(|e| MsgError::SendError(e.to_string()))?;
    Ok(())
}

pub async fn publish_data(payload: WsResponse) -> Result<()> {
    let stream_entities = get_stream_entities().await?;
    let json_bytes =
        serde_json::to_vec(&payload).map_err(|e| MsgError::SendError(e.to_string()))?;
    let msg = StreamMessage::builder()
        .properties()
        .content_type("application/json")
        .message_builder()
        .body(json_bytes)
        .build();

    stream_entities.producer.send_with_confirm(msg).await?;

    Ok(())
}
