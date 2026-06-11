use chrono::NaiveDateTime;
use dashmap::DashMap;
use mdcore::Ohlcv;
use rust_decimal::prelude::ToPrimitive;

use ta::indicators::{
    AverageTrueRange, BollingerBands, ExponentialMovingAverage, RelativeStrengthIndex,
    SimpleMovingAverage,
};
use ta::{DataItem, Next};

#[derive(Clone)]
pub struct IndicatorConfig {
    pub rsi_period: usize,
    pub ema_fast_period: usize,
    pub ema_slow_period: usize,
    pub macd_signal_period: usize,

    pub sma_period: usize,
    pub atr_period: usize,
    pub bb_period: usize,
    pub bb_std_dev: f64,

    pub roc_period: usize,
    pub vol_period: usize,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            ema_fast_period: 12,
            ema_slow_period: 26,
            macd_signal_period: 9,

            sma_period: 50,
            atr_period: 14,
            bb_period: 20,
            bb_std_dev: 2.0,

            roc_period: 10,
            vol_period: 20,
        }
    }
}

// ============================
// FEATURE OUTPUT
// ============================
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub symbol: String,
    pub timestamp: NaiveDateTime,

    pub rsi: f64,
    pub sma: f64,
    pub ema_fast: f64,
    pub atr: f64,

    pub bb_upper: f64,
    pub bb_middle: f64,
    pub bb_lower: f64,

    pub macd: f64,
    pub macd_signal: f64,
    pub macd_hist: f64,

    pub roc: f64,
    pub hist_vol: f64,
    pub vwap: f64,
}
pub struct Indicators {
    pub cfg: IndicatorConfig,

    // TA indicators
    pub rsi: RelativeStrengthIndex,
    pub sma: SimpleMovingAverage,
    pub ema_fast: ExponentialMovingAverage,
    pub ema_slow: ExponentialMovingAverage,
    pub ema_signal: ExponentialMovingAverage,
    pub atr: AverageTrueRange,
    pub bb: BollingerBands,

    // VWAP state
    pub vwap_pv: f64,
    pub vwap_vol: f64,

    // history buffers
    pub closes: Vec<f64>,
    pub log_returns: Vec<f64>,
}

impl Indicators {
    pub fn new(cfg: IndicatorConfig) -> Self {
        Self {
            rsi: RelativeStrengthIndex::new(cfg.rsi_period).unwrap(),
            sma: SimpleMovingAverage::new(cfg.sma_period).unwrap(),
            ema_fast: ExponentialMovingAverage::new(cfg.ema_fast_period).unwrap(),
            ema_slow: ExponentialMovingAverage::new(cfg.ema_slow_period).unwrap(),
            ema_signal: ExponentialMovingAverage::new(cfg.macd_signal_period).unwrap(),
            atr: AverageTrueRange::new(cfg.atr_period).unwrap(),
            bb: BollingerBands::new(cfg.bb_period, cfg.bb_std_dev).unwrap(),

            vwap_pv: 0.0,
            vwap_vol: 0.0,

            closes: Vec::new(),
            log_returns: Vec::new(),

            cfg,
        }
    }

    pub fn update(&mut self, symbol: &str, item: &Ohlcv) -> FeatureVector {
        let open = item.open.to_f64().unwrap_or(f64::NAN);
        let high = item.high.to_f64().unwrap_or(f64::NAN);
        let low = item.low.to_f64().unwrap_or(f64::NAN);
        let close = item.close.to_f64().unwrap_or(f64::NAN);
        let vol = item.volume.to_f64().unwrap_or(0.0);

        let di = DataItem::builder()
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(vol)
            .build()
            .unwrap();

        let typical = (high + low + close) / 3.0;
        self.vwap_pv += typical * vol;
        self.vwap_vol += vol;

        let vwap = if self.vwap_vol > 0.0 {
            self.vwap_pv / self.vwap_vol
        } else {
            f64::NAN
        };

        // ============================
        // MACD
        // ============================
        let fast = self.ema_fast.next(close);
        let slow = self.ema_slow.next(close);

        let macd = fast - slow;
        let signal = self.ema_signal.next(macd);
        let hist = macd - signal;

        // ============================
        // ROC
        // ============================
        self.closes.push(close);

        let roc = if self.closes.len() > self.cfg.roc_period {
            let prev = self.closes[self.closes.len() - self.cfg.roc_period - 1];
            if prev != 0.0 {
                (close / prev) - 1.0
            } else {
                f64::NAN
            }
        } else {
            f64::NAN
        };

        // ============================
        // Historical Volatility
        // ============================
        let hist_vol = if self.closes.len() > 1 {
            let prev = self.closes[self.closes.len() - 2];

            if prev > 0.0 && close > 0.0 {
                self.log_returns.push((close / prev).ln());
            }

            let n = self.cfg.vol_period;

            if self.log_returns.len() > n {
                let window = &self.log_returns[self.log_returns.len() - n..];

                let mean = window.iter().sum::<f64>() / n as f64;

                let var = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;

                var.sqrt()
            } else {
                f64::NAN
            }
        } else {
            f64::NAN
        };

        // ============================
        // OUTPUT
        // ============================
        FeatureVector {
            symbol: symbol.to_string(),
            timestamp: item.timestamp.naive_utc(),

            rsi: self.rsi.next(&di),
            sma: self.sma.next(&di),
            ema_fast: fast,
            atr: self.atr.next(&di),

            bb_upper: self.bb.next(&di).upper,
            bb_middle: self.bb.next(&di).average,
            bb_lower: self.bb.next(&di).lower,

            macd,
            macd_signal: signal,
            macd_hist: hist,

            roc,
            hist_vol,
            vwap,
        }
    }
}

pub fn run_backtest(mut state: Indicators, symbol: &str, data: Vec<Ohlcv>) -> Vec<FeatureVector> {
    data.iter().map(|c| state.update(symbol, c)).collect()
}

pub struct MultiSymbolEngine {
    pub cfg: IndicatorConfig,
    pub states: DashMap<String, Indicators>,
}

impl MultiSymbolEngine {
    pub fn new(cfg: IndicatorConfig) -> Self {
        Self {
            cfg,
            states: DashMap::new(),
        }
    }
}

impl MultiSymbolEngine {
    pub fn process(&self, symbol: &str, candle: &Ohlcv) -> FeatureVector {
        let mut state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| Indicators::new(self.cfg.clone()));

        state.update(symbol, candle)
    }
}
