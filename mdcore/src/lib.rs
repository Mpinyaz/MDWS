use chrono::{DateTime, NaiveDate, Utc};
use core::fmt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "camelCase")]
pub enum AssetClass {
    Crypto,
    Forex,
    Equity,
}

impl fmt::Debug for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl AssetClass {
    pub fn measurement(&self) -> &'static str {
        match self {
            AssetClass::Crypto => "crypto",
            AssetClass::Forex => "forex",
            AssetClass::Equity => "equity",
        }
    }
}

impl fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetClass::Crypto => write!(f, "crypto"),
            AssetClass::Forex => write!(f, "forex"),
            AssetClass::Equity => write!(f, "equity"),
        }
    }
}

impl FromStr for AssetClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "crypto" | "crypto_data" => Ok(AssetClass::Crypto),
            "forex" => Ok(AssetClass::Forex),
            "equity" => Ok(AssetClass::Equity),
            _ => Err(format!("Unknown AssetClass: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StdFrequency {
    #[serde(rename = "1min")]
    Minute,
    #[serde(rename = "5min")]
    Min5,
    #[serde(rename = "15min")]
    Min15,
    #[serde(rename = "30min")]
    Min30,
    #[serde(rename = "60min", alias = "1hour")]
    Hourly,
    #[serde(rename = "4hour")]
    Hour4,
    #[serde(rename = "12hour")]
    HalfDay,
    #[serde(rename = "24hour")]
    Day,
}

impl fmt::Display for StdFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StdFrequency::Minute => "1min",
            StdFrequency::Min5 => "5min",
            StdFrequency::Min15 => "15min",
            StdFrequency::Min30 => "30min",
            StdFrequency::Hourly => "60min",
            StdFrequency::Hour4 => "4hour",
            StdFrequency::HalfDay => "12hour",
            StdFrequency::Day => "24hour",
        };
        write!(f, "{}", s)
    }
}

// ── Equity Frequency ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquityFrequency {
    #[serde(rename = "hourly")]
    Hourly,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "yearly")]
    Yearly,
}

impl fmt::Display for EquityFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EquityFrequency::Hourly => "hourly",
            EquityFrequency::Daily => "daily",
            EquityFrequency::Weekly => "weekly",
            EquityFrequency::Monthly => "monthly",
            EquityFrequency::Yearly => "yearly",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Frequency {
    Equity(EquityFrequency),
    Std(StdFrequency),
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::Equity(freq) => write!(f, "{}", freq),
            Frequency::Std(freq) => write!(f, "{}", freq),
        }
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        if let Ok(freq) =
            serde_json::from_value::<EquityFrequency>(serde_json::Value::String(s.clone()))
        {
            return Ok(Frequency::Equity(freq));
        }

        if let Ok(freq) =
            serde_json::from_value::<StdFrequency>(serde_json::Value::String(s.clone()))
        {
            return Ok(Frequency::Std(freq));
        }

        Err(serde::de::Error::custom(format!(
            "'{}' is not a valid frequency.\n  \
            Equity accepts : hourly, daily, weekly, monthly, yearly\n  \
            Crypto | Forex accepts : 1min, 5min, 15min, 30min, 60min, 4hour, 12hour, 24hour",
            s
        )))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRequest {
    pub ticker: String,
    pub assetclass: AssetClass,
    pub datefrom: NaiveDate,
    pub dateto: NaiveDate,
    pub frequency: Frequency,
}
impl AssetRequest {
    pub fn validate_freq(&self) -> Result<(), String> {
        match self.assetclass {
            AssetClass::Equity => {
                if let Frequency::Std(_) = self.frequency {
                    return Err(format!(
                        "'{}' is not a valid frequency for Equity. Accepted: hourly, daily, weekly, monthly, yearly",
                        self.frequency
                    ));
                }
            }
            AssetClass::Crypto | AssetClass::Forex => {
                if let Frequency::Equity(_) = self.frequency {
                    return Err(format!(
                        "'{}' is not a valid frequency for {:?}. Accepted: 1min, 5min, 15min, 30min, 60min, 4hour, 12hour, 24hour",
                        self.frequency, self.assetclass
                    ));
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ohlcv {
    #[serde(alias = "date")]
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    #[serde(default)]
    pub volume: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarketMetadata {
    pub asset_class: AssetClass,
    pub frequency: Frequency,
}
