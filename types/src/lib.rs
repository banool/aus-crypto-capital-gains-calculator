use chrono::{DateTime, TimeZone};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Currency(pub String);

#[derive(Debug)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Transaction {
    // Amount of the transaction. Always a postive number.
    amount: f64,

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

impl Transaction {
    pub fn new(amount: f64, currency: Currency, rate: f64, transaction_type: TransactionType, unixtime: u64) -> Transaction {
        Transaction {
            amount,
            currency,
            rate,
            transaction_type,
            unixtime,
        }
    }
}
