use anyhow::Result;
use rabbitmq_stream_client::{error::StreamCreateError, NoDedup};
use rabbitmq_stream_client::{
    types::{ByteCapacity, ResponseCode, StreamCreator},
    Consumer,
};
use rabbitmq_stream_client::{Environment, Producer};
use tokio::sync::OnceCell;
use tracing::info;

use crate::config::cfg::Config;

pub struct StreamEntities {
    pub producer: Producer<NoDedup>,
    pub consumer: Consumer,
}

static STREAM_ENTITIES: OnceCell<StreamEntities> = OnceCell::const_new();

pub async fn get_stream_entities() -> Result<&'static StreamEntities> {
    STREAM_ENTITIES.get_or_try_init(init_stream_entities).await
}

async fn init_stream_entities() -> Result<StreamEntities> {
    let config = Config::load_env()?;
    let environment = Environment::builder()
        .host(&config.stream_host)
        .port(config.stream_port)
        .username(&config.stream_username)
        .password(&config.stream_password)
        .build()
        .await?;

    let updates_env = environment
        .stream_creator()
        .max_length(ByteCapacity::GB(5))
        .max_age(std::time::Duration::from_secs(3600 * 24 * 7));
    init_stream(updates_env, &config.feed_stream).await?;

    let subscribe_env = environment
        .stream_creator()
        .max_length(ByteCapacity::GB(2))
        .max_age(std::time::Duration::from_secs(3600 * 24 * 3));
    init_stream(subscribe_env, &config.subscribe_stream).await?;

    let producer = environment.producer().build(&config.feed_stream).await?;
    let consumer = environment
        .consumer()
        .build(&config.subscribe_stream)
        .await?;
    info!(
        "Stream entities initialized: Producer -> {}, Consumer -> {}",
        &config.feed_stream, &config.subscribe_stream
    );
    Ok(StreamEntities { producer, consumer })
}

async fn init_stream(builder: StreamCreator, stream_name: &String) -> Result<()> {
    match builder.create(stream_name).await {
        Ok(_) => info!("Stream '{}' created", stream_name),
        Err(StreamCreateError::Create {
            status: ResponseCode::StreamAlreadyExists | ResponseCode::PreconditionFailed,
            ..
        }) => {
            info!("Stream '{}' already exists", stream_name);
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
