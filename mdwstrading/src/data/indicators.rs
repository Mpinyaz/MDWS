use chrono::NaiveDateTime;
use mdcore::Ohlcv;
use polars::{df, error::PolarsResult, frame::DataFrame, prelude::Column};
use rust_decimal::prelude::ToPrimitive;
use ta::indicators::{
    AverageTrueRange, BollingerBands, ExponentialMovingAverage, RelativeStrengthIndex,
    SimpleMovingAverage,
};
use ta::{DataItem, Next};

pub struct IndicatorConfig {
    pub rsi_period: usize,
    pub ema_fast_period: usize,
    pub sma_period: usize,
    pub atr_period: usize,
    pub bb_period: usize,
    pub bb_std_dev: f64,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            sma_period: 50,
            ema_fast_period: 12,
            atr_period: 14,
            bb_period: 20,
            bb_std_dev: 2.0,
        }
    }
}

pub fn ohlcv_to_df(data: Vec<Ohlcv>) -> PolarsResult<DataFrame> {
    let mut ts = Vec::with_capacity(data.len());
    let mut open = Vec::with_capacity(data.len());
    let mut high = Vec::with_capacity(data.len());
    let mut low = Vec::with_capacity(data.len());
    let mut close = Vec::with_capacity(data.len());
    let mut vol = Vec::with_capacity(data.len());

    for item in data {
        ts.push(item.timestamp);
        open.push(item.open);
        high.push(item.high);
        low.push(item.low);
        close.push(item.close);
        vol.push(item.volume);
    }
    let naive_ts: Vec<NaiveDateTime> = ts.into_iter().map(|dt| dt.naive_utc()).collect();
    let open_f64: Vec<f64> = open
        .into_iter()
        .map(|d| d.to_f64().unwrap_or(f64::NAN))
        .collect();
    let close_f64: Vec<f64> = close
        .into_iter()
        .map(|d| d.to_f64().unwrap_or(f64::NAN))
        .collect();
    let high_f64: Vec<f64> = high
        .into_iter()
        .map(|d| d.to_f64().unwrap_or(f64::NAN))
        .collect();
    let low_f64: Vec<f64> = low
        .into_iter()
        .map(|d| d.to_f64().unwrap_or(f64::NAN))
        .collect();
    let vol_f64: Vec<f64> = vol
        .into_iter()
        .map(|d| d.to_f64().unwrap_or(f64::NAN))
        .collect();
    df!(
        "timestamp" => naive_ts,
        "open" => open_f64,
        "high" => high_f64,
        "low" => low_f64,
        "close" => close_f64,
        "volume" => vol_f64,
    )
}

pub fn compute_indicators(data: Vec<Ohlcv>, cfg: IndicatorConfig) -> PolarsResult<DataFrame> {
    let df = ohlcv_to_df(data.clone())?;

    // Use config values for initialization
    let mut sma = SimpleMovingAverage::new(cfg.sma_period).unwrap();
    let mut rsi = RelativeStrengthIndex::new(cfg.rsi_period).unwrap();
    let mut ema_fast = ExponentialMovingAverage::new(cfg.ema_fast_period).unwrap();
    let mut atr = AverageTrueRange::new(cfg.atr_period).unwrap();
    let mut bb = BollingerBands::new(cfg.bb_period, cfg.bb_std_dev).unwrap();

    let mut rsi_vals = Vec::with_capacity(data.len());
    let mut ema_vals = Vec::with_capacity(data.len());
    let mut sma_vals = Vec::with_capacity(data.len());
    let mut atr_vals = Vec::with_capacity(data.len());
    let mut bb_upper = Vec::with_capacity(data.len());
    let mut bb_middle = Vec::with_capacity(data.len());
    let mut bb_lower = Vec::with_capacity(data.len());

    for item in data {
        let out = DataItem::builder()
            .open(item.open.to_f64().unwrap_or(f64::NAN))
            .high(item.high.to_f64().unwrap_or(f64::NAN))
            .low(item.low.to_f64().unwrap_or(f64::NAN))
            .close(item.close.to_f64().unwrap_or(f64::NAN))
            .volume(item.volume.to_f64().unwrap_or(f64::NAN))
            .build()
            .unwrap();

        let bb_out = bb.next(&out);
        bb_upper.push(bb_out.upper);
        bb_middle.push(bb_out.average);
        bb_lower.push(bb_out.lower);

        rsi_vals.push(rsi.next(&out));
        ema_vals.push(ema_fast.next(&out));
        sma_vals.push(sma.next(&out));
        atr_vals.push(atr.next(&out));
    }

    df.hstack(&[
        Column::new("rsi".into(), rsi_vals),
        Column::new(format!("ema_{}", cfg.ema_fast_period).into(), ema_vals),
        Column::new("atr".into(), atr_vals),
        Column::new(format!("sma_{}", cfg.sma_period).into(), sma_vals),
        Column::new("bb_upper".into(), bb_upper),
        Column::new("bb_middle".into(), bb_middle),
        Column::new("bb_lower".into(), bb_lower),
    ])
}
