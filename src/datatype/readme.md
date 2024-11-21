# Datatype

- [Datatype](#datatype)
  - [Position](#position)

## Position

recommended Position
- **account_id:** Useful if you're managing positions across multiple accounts.
- **instrument:** Specifies the asset being traded, such as a stock symbol or currency pair.
- **position_type:** Indicates whether the position is a long or short.
- **leverage:** If trading on margin, this represents the leverage ratio.
- **pnl (Profit and Loss):** While this can be calculated from entry and exit prices, storing it can simplify reporting and analytics.
- **fees:** To account for any commissions or transaction fees associated with the trade.
- **currency:** Specifies the currency in which the trade is denominated, important for multi-currency portfolios.
- **entry_order_id and exit_order_id:** Helps link the position to the specific orders that opened and closed it.
- **tags:** Allows for flexible metadata, such as tagging positions with strategy names or risk categories.

```rs
use chrono::{DateTime, Utc};

pub enum PositionType {
    Long,
    Short,
}

pub enum OrderType {
    Market,
    Limit,
    Stop,
    // Add other order types as needed
}

pub enum PositionStatus {
    Open,
    Closed,
    Pending,
    PartiallyFilled
    // Add other statuses as needed
}

pub struct Position {
    pub id: u32,
    pub account_id: u32,                  // To identify the trading account
    pub instrument: String,               // The asset being traded (e.g., stock symbol)
    pub position_type: PositionType,      // Long or Short
    pub entry_datetime: DateTime<Utc>,    // Using chrono for accurate timestamps
    pub exit_datetime: Option<DateTime<Utc>>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: PositionStatus,
    pub volume: f64,                      // Number of units traded
    pub leverage: Option<f64>,            // Leverage used, if any
    pub pnl: Option<f64>,                 // Profit and Loss
    pub fees: Option<f64>,                // Commissions or fees paid
    pub currency: String,                 // Currency of the position
    pub entry_order_id: Option<u32>,      // ID of the entry order
    pub exit_order_id: Option<u32>,       // ID of the exit order
    pub tags: Option<Vec<String>>,        // For additional metadata (e.g., strategy tags)
}
```