use mdcore::AssetClass;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub enum SlippageModel {
    Fixed { bps: Decimal },
    Linear { impact_rate: Decimal },
    SquareRoot { eta: Decimal },
}

impl SlippageModel {
    /// Returns dollar impact of the trade.
    /// Buy orders pay more, sell orders receive less — both positive cost.
    pub fn estimate(
        &self,
        quantity: Decimal,
        price: Decimal,
        adv: Decimal,
        daily_vol: Decimal,
        asset_class: &AssetClass,
    ) -> Decimal {
        if adv == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let notional = quantity.abs() * price;
        let participation = quantity.abs() / adv;

        let slip_fraction = match self {
            SlippageModel::Fixed { bps } => bps / dec!(10_000),
            SlippageModel::Linear { impact_rate } => impact_rate * participation,
            SlippageModel::SquareRoot { eta } => {
                let sqrt_part = participation
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .sqrt();
                let sigma = daily_vol.to_string().parse::<f64>().unwrap_or(0.015);
                let eta_f = eta.to_string().parse::<f64>().unwrap_or(0.1);
                Decimal::try_from(sigma * eta_f * sqrt_part).unwrap_or_default()
            }
        };

        let max_slip = match asset_class {
            AssetClass::Equity => dec!(0.005),
            AssetClass::Crypto => dec!(0.015),
            AssetClass::Forex => dec!(0.001),
        };

        // Dollar impact = capped fractional slippage × notional
        slip_fraction.min(max_slip) * notional
    }

    /// Returns the fill price after applying dollar impact.
    pub fn apply_to_price(
        &self,
        mid_price: Decimal,
        is_buy: bool,
        quantity: Decimal,
        adv: Decimal,
        daily_vol: Decimal,
        asset_class: &AssetClass,
    ) -> Decimal {
        if quantity == Decimal::ZERO {
            return mid_price;
        }

        let dollar_impact = self.estimate(quantity, mid_price, adv, daily_vol, asset_class);

        // Convert back to per-unit price adjustment
        let price_adjustment = dollar_impact / quantity.abs();

        if is_buy {
            mid_price + price_adjustment
        } else {
            mid_price - price_adjustment
        }
    }

    pub fn for_equity() -> Self {
        SlippageModel::SquareRoot { eta: dec!(0.10) }
    }
    pub fn for_crypto() -> Self {
        SlippageModel::Linear {
            impact_rate: dec!(0.15),
        }
    }
    pub fn for_forex() -> Self {
        SlippageModel::Fixed { bps: dec!(0.5) }
    }
}
