use influxdb::{Client, ReadQuery};
use mdanalytics::json_to_dataframe;
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let url = std::env::var("INFLUXDB_URL").expect("database url is required");
    let token = std::env::var("INFLUXDB_TOKEN").expect("Token is rquired");
    let client = Client::new(url, "marketupdates").with_token(token);

    // --- 1. CRYPTO ---
    let crypto_raw = client.query(ReadQuery::new("SELECT * FROM crypto")).await?;
    let df_crypto = json_to_dataframe(&crypto_raw, &["last_price", "last_size"])?;

    let crypto_stats = df_crypto
        .lazy()
        .group_by([col("ticker")])
        .agg([
            col("time").min().alias("start_time"),
            col("time").max().alias("end_time"),
            col("last_price").count().alias("data_points"),
            col("last_price").mean().alias("avg_price"),
        ])
        .collect()?;

    // --- 2. FOREX ---
    let forex_raw = client.query(ReadQuery::new("SELECT * FROM forex")).await?;
    let df_forex = json_to_dataframe(&forex_raw, &["bid_price", "ask_price", "mid_price"])?;

    let forex_stats = df_forex
        .lazy()
        .group_by([col("ticker")])
        .agg([
            col("time").min().alias("start_time"),
            col("mid_price").count().alias("data_points"),
            col("mid_price").mean().alias("avg_mid"),
        ])
        .collect()?;

    println!("=== CRYPTO STATS ===\n{}", crypto_stats);
    println!("=== FOREX STATS ===\n{}", forex_stats);

    Ok(())
}
