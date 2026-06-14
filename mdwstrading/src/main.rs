use mdcore::{AssetClass, AssetRequest, Ohlcv};

use reqwest::Client;

use mdwstrading::data::indicators::{IndicatorConfig, IndicatorsEngine, ReturnStats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("MDANALYTICS_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let url = format!("{}/fetch/ohlcv", base_url);

    let client = Client::new();

    let payload = AssetRequest {
        ticker: "BTCUSD".to_string(),
        frequency: "15min".into(),

        assetclass: AssetClass::Crypto,

        datefrom: "2025-01-01".parse()?,

        dateto: "2026-06-10".parse()?,
    };

    let response = client.post(url).json(&payload).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("Server returned status {}: {}", status, err_text).into());
    }
    let candles: Vec<Ohlcv> = response.json().await?;

    let engine = IndicatorsEngine::new(IndicatorConfig::default());

    let mut features_list = Vec::new();

    for candle in &candles {
        let features = engine.process(&payload.ticker, &candle);

        println!("{:#?}", features);
        features_list.push(features);
    }

    if !features_list.is_empty() {
        let mut rsi_sum = 0.0;
        let mut rsi_count = 0;
        let mut sma_sum = 0.0;
        let mut sma_count = 0;
        let mut ema_fast_sum = 0.0;
        let mut ema_fast_count = 0;
        let mut atr_sum = 0.0;
        let mut atr_count = 0;
        let mut bb_upper_sum = 0.0;
        let mut bb_upper_count = 0;
        let mut bb_middle_sum = 0.0;
        let mut bb_middle_count = 0;
        let mut bb_lower_sum = 0.0;
        let mut bb_lower_count = 0;
        let mut macd_sum = 0.0;
        let mut macd_count = 0;
        let mut macd_signal_sum = 0.0;
        let mut macd_signal_count = 0;
        let mut macd_hist_sum = 0.0;
        let mut macd_hist_count = 0;
        let mut roc_sum = 0.0;
        let mut roc_count = 0;
        let mut hist_vol_sum = 0.0;
        let mut hist_vol_count = 0;
        let mut vwap_sum = 0.0;
        let mut vwap_count = 0;

        for f in &features_list {
            if !f.rsi.is_nan() {
                rsi_sum += f.rsi;
                rsi_count += 1;
            }
            if !f.sma.is_nan() {
                sma_sum += f.sma;
                sma_count += 1;
            }
            if !f.ema_fast.is_nan() {
                ema_fast_sum += f.ema_fast;
                ema_fast_count += 1;
            }
            if !f.atr.is_nan() {
                atr_sum += f.atr;
                atr_count += 1;
            }
            if !f.bb_upper.is_nan() {
                bb_upper_sum += f.bb_upper;
                bb_upper_count += 1;
            }
            if !f.bb_middle.is_nan() {
                bb_middle_sum += f.bb_middle;
                bb_middle_count += 1;
            }
            if !f.bb_lower.is_nan() {
                bb_lower_sum += f.bb_lower;
                bb_lower_count += 1;
            }
            if !f.macd.is_nan() {
                macd_sum += f.macd;
                macd_count += 1;
            }
            if !f.macd_signal.is_nan() {
                macd_signal_sum += f.macd_signal;
                macd_signal_count += 1;
            }
            if !f.macd_hist.is_nan() {
                macd_hist_sum += f.macd_hist;
                macd_hist_count += 1;
            }
            if !f.roc.is_nan() {
                roc_sum += f.roc;
                roc_count += 1;
            }
            if !f.hist_vol.is_nan() {
                hist_vol_sum += f.hist_vol;
                hist_vol_count += 1;
            }
            if !f.vwap.is_nan() {
                vwap_sum += f.vwap;
                vwap_count += 1;
            }
        }

        println!("\n=================================");
        println!(
            "AVERAGE INDICATOR VALUES OVER {} DATA POINTS",
            features_list.len()
        );
        println!("=================================");
        println!(
            "RSI:                 {:.4}",
            if rsi_count > 0 {
                rsi_sum / rsi_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "SMA:                 {:.4}",
            if sma_count > 0 {
                sma_sum / sma_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "EMA Fast:            {:.4}",
            if ema_fast_count > 0 {
                ema_fast_sum / ema_fast_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "ATR:                 {:.4}",
            if atr_count > 0 {
                atr_sum / atr_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "BB Upper:            {:.4}",
            if bb_upper_count > 0 {
                bb_upper_sum / bb_upper_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "BB Middle:           {:.4}",
            if bb_middle_count > 0 {
                bb_middle_sum / bb_middle_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "BB Lower:            {:.4}",
            if bb_lower_count > 0 {
                bb_lower_sum / bb_lower_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "MACD:                {:.4}",
            if macd_count > 0 {
                macd_sum / macd_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "MACD Signal:         {:.4}",
            if macd_signal_count > 0 {
                macd_signal_sum / macd_signal_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "MACD Histogram:      {:.4}",
            if macd_hist_count > 0 {
                macd_hist_sum / macd_hist_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "ROC:                 {:.6}",
            if roc_count > 0 {
                roc_sum / roc_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "Historical Vol:      {:.6}",
            if hist_vol_count > 0 {
                hist_vol_sum / hist_vol_count as f64
            } else {
                f64::NAN
            }
        );
        println!(
            "VWAP:                {:.4}",
            if vwap_count > 0 {
                vwap_sum / vwap_count as f64
            } else {
                f64::NAN
            }
        );
        println!("=================================");

        if let Some(stats) = ReturnStats::calculate(
            payload.ticker,
            payload.frequency,
            payload.assetclass,
            &candles,
        ) {
            println!("\n=================================");
            println!("RETURN STATISTICS FROM LOG RETURNS");
            println!("=================================");
            println!("Count:        {}", stats.count);
            println!("Mean:         {:.6}", stats.mean);
            println!("Median:       {:.6}", stats.median);
            println!("StdDev:       {:.6}", stats.stddev);
            println!("Min:          {:.6}", stats.min);
            println!("Max:          {:.6}", stats.max);
            println!("p01:          {:.6}", stats.p01);
            println!("p05:          {:.6}", stats.p05);
            println!("p25:          {:.6}", stats.p25);
            println!("p50:          {:.6}", stats.p50);
            println!("p75:          {:.6}", stats.p75);
            println!("p95:          {:.6}", stats.p95);
            println!("p99:          {:.6}", stats.p99);
            println!("Positive Pct: {:.2}%", stats.positive_pct * 100.0);
            println!("Negative Pct: {:.2}%", stats.negative_pct * 100.0);
            println!("=================================");
        } else {
            println!("\nCould not calculate return statistics (insufficient data).");
        }
    }

    Ok(())
}
