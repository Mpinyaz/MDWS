use mdworkers::types::asset::AssetClass;
use mdworkers::{config::cfg::Config, models::client::Client};
use tokio::net::TcpStream;
use tracing::{info, warn};

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = Config::load_env()?;

    let client = Client::new(&cfg);

    Ok(())
}
