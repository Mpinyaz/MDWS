use core::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum AssetClass {
    Crypto,
    Forex,
    Equity,
}

impl fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetClass::Crypto => write!(f, "crypto"),
            AssetClass::Forex => write!(f, "fx"),
            AssetClass::Equity => write!(f, "iex"),
        }
    }
}
impl fmt::Debug for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<&str> for AssetClass {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "crypto" => Ok(AssetClass::Crypto),
            "forex" => Ok(AssetClass::Forex),
            "equity" => Ok(AssetClass::Equity),
            _ => Err(format!("Unknown asset class: {}", s)),
        }
    }
}

impl TryFrom<&String> for AssetClass {
    type Error = String;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        AssetClass::try_from(s.as_str())
    }
}
