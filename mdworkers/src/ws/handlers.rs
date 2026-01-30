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
use tracing::error;

pub async fn outbound_msg_handler<F>(
    mut client: Client,
    asset_class: AssetClass,
    mut on_message: F,
) -> Result<(), MsgError>
where
    F: FnMut(Message) -> Result<WsResponse, MsgError>,
{
    let conn = match asset_class {
        AssetClass::Forex => client.fx_ws.take(),
        AssetClass::Crypto => client.crypto_ws.take(),
        AssetClass::Equity => client.equity_ws.take(),
    }
    .ok_or_else(|| MsgError::ReadError(format!("{:?} not connected", asset_class)))?;

    let (_, mut read) = conn.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => match on_message(message) {
                Ok(ws_response) => {
                    if let Err(e) = publish_data(ws_response).await {
                        error!("Publish failed: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error handling message: {}", e);
                    return Err(e);
                }
            },
            Err(e) => {
                return Err(MsgError::ReadError(format!("WebSocket error: {}", e)));
            }
        }
    }
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
