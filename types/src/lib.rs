use chrono::{DateTime, TimeZone};

#[derive(Clone, Debug)]
pub struct Currency(pub String);

pub enum TransactionType {
    Buy,
    Sell,
}

pub struct Transaction {
    // Amount of the transaction. Always a postive number.
    amount: u64,

    // Currency the transaction was made in.
    // The other side of the transaction is assumed to be AUD.
    currency: Currency,

    // Conversion rate of currency to AUD.
    // e.g. If 1 BTC costs 40,000 AUD, this would be 40,000.
    rate: f64,

    // Whether this was a buy or sell.
    transaction_type: TransactionType,

    // When the transaction was made.
    pub unixtime: u64,
}
