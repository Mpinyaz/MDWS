use mdcore::{AssetClass, AssetRequest, Ohlcv};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://mdanalytics:3030/fetch/ohlcv";

    let client = Client::new();

    let payload = AssetRequest {
        ticker: "AAPL".to_string(),
        assetclass: AssetClass::Equity,
        datefrom: "2024-01-01".parse().unwrap(),
        dateto: "2024-12-31".parse().unwrap(),
    };

    let response: Vec<Ohlcv> = client
        .post(url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", response);

    Ok(())
}
