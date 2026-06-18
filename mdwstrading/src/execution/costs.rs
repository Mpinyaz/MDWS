use mdcore::AssetClass;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct TransactionCosts {
    pub equity: EquityCosts,
    pub crypto: CryptoCosts,
    pub forex: ForexCosts,
}

#[derive(Debug, Clone)]
pub struct EquityCosts {
    /// Per-share commission, e.g. dec!(0.005) = $0.005/share
    pub commission_per_share: Decimal,
    /// Minimum ticket charge, e.g. dec!(1.00)
    pub min_commission: Decimal,
    /// SEC/FINRA regulatory fee on sells: ~$0.000166 per dollar of proceeds
    pub regulatory_fee_rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct CryptoCosts {
    /// Taker fee as fraction of notional, e.g. dec!(0.001) = 10bps
    pub taker_fee: Decimal,
    /// Maker fee — lower, rewarded for adding liquidity
    pub maker_fee: Decimal,
    /// Whether this order is likely to take liquidity (market order = true)
    pub is_taker: bool,
}

#[derive(Debug, Clone)]
pub struct ForexCosts {
    /// Half-spread in pips, e.g. dec!(0.5) for EUR/USD
    pub half_spread_pips: Decimal,
    /// Pip value per lot in base currency, e.g. dec!(10) for EUR/USD standard lot
    pub pip_value: Decimal,
    /// Overnight swap/rollover rate (annualised), applied if position held past cutoff
    pub swap_rate: Decimal,
}

impl TransactionCosts {
    /// Returns a typical retail broker cost model
    pub fn retail_defaults() -> Self {
        TransactionCosts {
            equity: EquityCosts {
                commission_per_share: dec!(0.005),
                min_commission: dec!(1.00),
                regulatory_fee_rate: dec!(0.000166),
            },
            crypto: CryptoCosts {
                taker_fee: dec!(0.001), // 10bps — Binance spot taker
                maker_fee: dec!(0.0004),
                is_taker: true,
            },
            forex: ForexCosts {
                half_spread_pips: dec!(0.6), // EUR/USD retail spread ~1.2 pips
                pip_value: dec!(10.0),       // per standard lot
                swap_rate: dec!(0.02),       // ~2% annualised
            },
        }
    }

    /// Total round-trip cost for a trade (entry + exit combined)
    pub fn round_trip_cost(
        &self,
        asset_class: &AssetClass,
        quantity: Decimal,
        price: Decimal,
    ) -> Decimal {
        let notional = quantity.abs() * price;
        match asset_class {
            AssetClass::Equity => {
                let comm = (quantity.abs() * self.equity.commission_per_share)
                    .max(self.equity.min_commission);
                let reg = notional * self.equity.regulatory_fee_rate;
                (comm + reg) * dec!(2) // ×2 for round trip
            }
            AssetClass::Crypto => {
                let fee = if self.crypto.is_taker {
                    self.crypto.taker_fee
                } else {
                    self.crypto.maker_fee
                };
                notional * fee * dec!(2)
            }
            AssetClass::Forex => {
                // Spread cost = 2 × half_spread × pip_value × lots
                let lot_size = dec!(100_000);
                let lots = quantity.abs() / lot_size;
                let full_spread = self.forex.half_spread_pips * dec!(2);
                full_spread * self.forex.pip_value * lots
            }
        }
    }

    /// One-way cost for a single fill (used in backtesting)
    pub fn one_way_cost(
        &self,
        asset_class: &AssetClass,
        quantity: Decimal,
        price: Decimal,
    ) -> (Decimal, Decimal) {
        // Returns (commission, spread_cost)
        let notional = quantity.abs() * price;
        match asset_class {
            AssetClass::Equity => {
                let comm = (quantity.abs() * self.equity.commission_per_share)
                    .max(self.equity.min_commission);
                let spread = notional * dec!(0.0001); // ~1bp half-spread for liquid stocks
                (comm, spread)
            }
            AssetClass::Crypto => {
                let fee = if self.crypto.is_taker {
                    self.crypto.taker_fee
                } else {
                    self.crypto.maker_fee
                };
                (notional * fee, Decimal::ZERO)
            }
            AssetClass::Forex => {
                let lot_size = dec!(100_000);
                let lots = quantity.abs() / lot_size;
                let spread = self.forex.half_spread_pips * self.forex.pip_value * lots;
                (Decimal::ZERO, spread) // forex has no explicit commission
            }
        }
    }
}
