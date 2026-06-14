use chrono::NaiveDateTime;
use dashmap::DashMap;
use mdcore::{AssetClass, Ohlcv};
use rust_decimal::prelude::ToPrimitive;

pub fn get_annualization_factor(frequency: &str, asset_class: AssetClass) -> f64 {
    let days_per_year = match asset_class {
        AssetClass::Crypto => 365.0,
        AssetClass::Forex => 260.0,
        AssetClass::Equity => 252.0,
    };

    let periods_per_day = match frequency.to_lowercase().as_str() {
        "daily" | "1day" => 1.0,
        "hourly" | "1hour" | "60min" => match asset_class {
            AssetClass::Equity => 6.5,
            _ => 24.0,
        },
        "15min" => match asset_class {
            AssetClass::Equity => 6.5 * 4.0,
            _ => 24.0 * 4.0,
        },
        "5min" => match asset_class {
            AssetClass::Equity => 6.5 * 12.0,
            _ => 24.0 * 12.0,
        },
        "1min" => match asset_class {
            AssetClass::Equity => 6.5 * 60.0,
            _ => 24.0 * 60.0,
        },
        _ => 1.0,
    };

    days_per_year * periods_per_day
}

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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

pub struct IndicatorsEngine {
    pub cfg: IndicatorConfig,
    pub states: DashMap<String, Indicators>,
}

impl IndicatorsEngine {
    pub fn new(cfg: IndicatorConfig) -> Self {
        Self {
            cfg,
            states: DashMap::new(),
        }
    }

    pub fn process(&self, symbol: &str, candle: &Ohlcv) -> FeatureVector {
        let mut state = self
            .states
            .entry(symbol.to_string())
            .or_insert_with(|| Indicators::new(self.cfg.clone()));

        state.update(symbol, candle)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReturnStats {
    pub symbol: String,
    pub count: usize,

    pub mean: f64,
    pub median: f64,

    pub stddev: f64,
    pub skewness: f64,
    pub kurtosis: f64,

    pub annualized_volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,

    pub min: f64,
    pub max: f64,

    pub p01: f64,
    pub p05: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
    pub p99: f64,

    pub var_95: f64,
    pub cvar_95: f64,

    pub positive_pct: f64,
    pub negative_pct: f64,
}

impl ReturnStats {
    pub fn calculate(
        symbol: String,
        freq: String,
        assetclass: AssetClass,
        ohlcv: &[Ohlcv],
    ) -> Option<Self> {
        if ohlcv.len() < 2 {
            return None;
        }

        let mut returns = Vec::with_capacity(ohlcv.len() - 1);
        for i in 1..ohlcv.len() {
            if let (Some(prev), Some(curr)) = (ohlcv[i - 1].close.to_f64(), ohlcv[i].close.to_f64())
            {
                if prev > 0.0 && curr > 0.0 {
                    returns.push((curr / prev).ln());
                }
            }
        }

        if returns.is_empty() {
            return None;
        }

        let count = returns.len();
        let sum: f64 = returns.iter().sum();
        let mean = sum / count as f64;

        let stddev = if count > 1 {
            let variance =
                returns.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (count - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // Sort to compute percentiles/median
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = returns[0];
        let max = returns[count - 1];

        let percentile = |sorted: &[f64], p: f64| -> f64 {
            let idx = p * (sorted.len() - 1) as f64;
            let idx_low = idx.floor() as usize;
            let idx_high = idx.ceil() as usize;
            if idx_low == idx_high {
                sorted[idx_low]
            } else {
                let weight = idx - idx_low as f64;
                sorted[idx_low] + weight * (sorted[idx_high] - sorted[idx_low])
            }
        };

        let median = percentile(&returns, 0.5);
        let p01 = percentile(&returns, 0.01);
        let p05 = percentile(&returns, 0.05);
        let p25 = percentile(&returns, 0.25);
        let p50 = percentile(&returns, 0.50);
        let p75 = percentile(&returns, 0.75);
        let p95 = percentile(&returns, 0.95);
        let p99 = percentile(&returns, 0.99);

        let skewness = if count > 2 && stddev > 0.0 {
            let sum_cubed_diff: f64 = returns.iter().map(|&x| (x - mean).powi(3)).sum();
            (sum_cubed_diff / count as f64) / stddev.powi(3)
        } else {
            0.0
        };

        let kurtosis = if count > 3 && stddev > 0.0 {
            let sum_fourth_diff: f64 = returns.iter().map(|&x| (x - mean).powi(4)).sum();
            (sum_fourth_diff / count as f64) / stddev.powi(4)
        } else {
            0.0
        };

        let ann_factor = get_annualization_factor(&freq, assetclass);
        let annualized_volatility = stddev * ann_factor;

        let sharpe_ratio = if stddev > 0.0 {
            (mean / stddev) * ann_factor
        } else {
            0.0
        };

        // Max Drawdown calculation from prices
        let mut max_drawdown = 0.0;
        let mut peak = -1.0;
        for item in ohlcv {
            if let Some(price) = item.close.to_f64() {
                if price > peak {
                    peak = price;
                } else if peak > 0.0 {
                    let dd = (peak - price) / peak;
                    if dd > max_drawdown {
                        max_drawdown = dd;
                    }
                }
            }
        }

        let var_95 = (-p05).max(0.0);
        let tail_returns: Vec<f64> = returns.iter().copied().filter(|&r| r <= p05).collect();
        let cvar_95 = if !tail_returns.is_empty() {
            let avg_tail_return = tail_returns.iter().sum::<f64>() / tail_returns.len() as f64;
            (-avg_tail_return).max(0.0)
        } else {
            var_95
        };

        let pos_count = returns.iter().filter(|&&x| x > 0.0).count();
        let neg_count = returns.iter().filter(|&&x| x < 0.0).count();
        let positive_pct = pos_count as f64 / count as f64;
        let negative_pct = neg_count as f64 / count as f64;

        Some(Self {
            symbol,
            count,
            mean,
            median,
            stddev,
            skewness,
            kurtosis,
            annualized_volatility,
            sharpe_ratio,
            max_drawdown,
            min,
            max,
            p01,
            p05,
            p25,
            p50,
            p75,
            p95,
            p99,
            var_95,
            cvar_95,
            positive_pct,
            negative_pct,
        })
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TickerStats {
    pub descriptive: ReturnStats,
    pub feature: FeatureVector,
}

impl TickerStats {
    pub fn calculate(
        symbol: String,
        freq: String,
        assetclass: AssetClass,
        ohlcv: &[Ohlcv],
    ) -> Option<Self> {
        let descriptive = ReturnStats::calculate(symbol.clone(), freq, assetclass, ohlcv)?;

        let mut indicators = Indicators::new(IndicatorConfig::default());
        let mut last_feature = None;
        for item in ohlcv {
            last_feature = Some(indicators.update(&symbol, item));
        }

        Some(Self {
            descriptive,
            feature: last_feature?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[test]
    fn test_return_stats_calculate() {
        let now = Utc::now();
        let ohlcv = vec![
            Ohlcv {
                timestamp: now,
                open: dec!(100.0),
                high: dec!(100.0),
                low: dec!(100.0),
                close: dec!(100.0),
                volume: dec!(1000.0),
            },
            Ohlcv {
                timestamp: now,
                open: dec!(105.0),
                high: dec!(105.0),
                low: dec!(105.0),
                close: dec!(105.0),
                volume: dec!(1000.0),
            },
            Ohlcv {
                timestamp: now,
                open: dec!(102.0),
                high: dec!(102.0),
                low: dec!(102.0),
                close: dec!(102.0),
                volume: dec!(1000.0),
            },
        ];

        let stats =
            ReturnStats::calculate("Dummy".into(), "15min".into(), AssetClass::Crypto, &ohlcv)
                .unwrap();
        assert_eq!(stats.count, 2);

        let r1 = (105.0 / 100.0f64).ln();
        let r2 = (102.0 / 105.0f64).ln();

        let expected_mean = (r1 + r2) / 2.0;
        assert!((stats.mean - expected_mean).abs() < 1e-9);

        // sample standard deviation
        let expected_stddev =
            (((r1 - expected_mean).powi(2) + (r2 - expected_mean).powi(2)) / 1.0f64).sqrt();
        assert!((stats.stddev - expected_stddev).abs() < 1e-9);

        assert_eq!(stats.min, r2.min(r1));
        assert_eq!(stats.max, r2.max(r1));

        assert!((stats.positive_pct - 0.5).abs() < 1e-9);
        assert!((stats.negative_pct - 0.5).abs() < 1e-9);

        assert!(stats.skewness.is_finite());
        assert!(stats.kurtosis.is_finite());
        assert!(stats.annualized_volatility >= 0.0);
        assert!(stats.sharpe_ratio.is_finite());
        assert!(stats.max_drawdown >= 0.0);
        assert!(stats.var_95 >= 0.0);
        assert!(stats.cvar_95 >= 0.0);
    }

    #[test]
    fn test_indicator_stats_calculate() {
        let now = Utc::now();
        let ohlcv = vec![
            Ohlcv {
                timestamp: now,
                open: dec!(100.0),
                high: dec!(101.0),
                low: dec!(99.0),
                close: dec!(100.5),
                volume: dec!(1000.0),
            },
            Ohlcv {
                timestamp: now,
                open: dec!(100.5),
                high: dec!(102.0),
                low: dec!(100.0),
                close: dec!(101.5),
                volume: dec!(1100.0),
            },
            Ohlcv {
                timestamp: now,
                open: dec!(101.5),
                high: dec!(101.5),
                low: dec!(98.0),
                close: dec!(99.0),
                volume: dec!(1200.0),
            },
        ];

        let stats =
            TickerStats::calculate("TEST".into(), "1min".into(), AssetClass::Crypto, &ohlcv)
                .unwrap();
        assert_eq!(stats.descriptive.symbol, "TEST");
        assert_eq!(stats.descriptive.count, 2);
        assert_eq!(stats.feature.symbol, "TEST");
        assert!(stats.feature.rsi.is_finite());
        assert!(stats.feature.sma.is_finite());
        assert!(stats.feature.ema_fast > 0.0);
    }
}
