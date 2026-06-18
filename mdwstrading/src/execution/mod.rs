pub mod costs;
pub mod slippage;
use mdcore::AssetClass;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub quantity: Decimal, // positive = buy, negative = sell
    pub limit_price: Option<Decimal>,
    pub order_type: OrderType,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
    StopLimit,
}

#[derive(Debug, Clone)]
pub struct FillResult {
    pub order: Order,
    pub filled_qty: Decimal,
    pub fill_price: Decimal,
    pub commission: Decimal,
    pub spread_cost: Decimal,
    pub market_impact: Decimal, // market impact component
    pub total_cost: Decimal,
}

impl FillResult {
    pub fn net_proceeds(&self) -> Decimal {
        // For a buy:  -(filled_qty * fill_price) - total_cost
        // For a sell: +(filled_qty * fill_price) - total_cost
        let gross = self.filled_qty * self.fill_price;
        if self.order.quantity > Decimal::ZERO {
            -gross - self.total_cost
        } else {
            gross - self.total_cost
        }
    }
}
