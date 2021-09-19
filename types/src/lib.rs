use chrono::{DateTime, TimeZone};
use log::debug;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Currency(pub String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub struct Transaction {
    // Amount of the transaction. Always a postive number.
    amount: f64,

    // Currency the transaction was made in.
    // The other side of the transaction is assumed to be AUD.
    pub currency: Currency,

    // Conversion rate of currency to AUD.
    // e.g. If 1 BTC costs 40,000 AUD, this would be 40,000.
    rate: f64,

    // Whether this was a buy or sell.
    pub transaction_type: TransactionType,

    // When the transaction was made.
    pub unixtime: u64,
}

impl Transaction {
    pub fn new(
        amount: f64,
        currency: Currency,
        rate: f64,
        transaction_type: TransactionType,
        unixtime: u64,
    ) -> Transaction {
        Transaction {
            amount,
            currency,
            rate,
            transaction_type,
            unixtime,
        }
    }

    /// Returns true if this transaction has nothing left in it.
    /// We check for less than a small amount instead of 0 to deal
    /// with floating point arithmetic inaccuracy.
    pub fn is_exhausted(&self) -> bool {
        self.amount < 0.00001
    }

    ///  Return from this function: (
    ///      capital gain from this sale,
    ///      maybe this Transaction depending on if the sale depleted it
    ///      maybe the other Transaction depending on if it is depleted
    /// )
    ///
    /// This function assumes that this Transaction (self) is a buy, and then takes
    /// in another Transaction assuming it is a sell, and figures out how much is
    /// left of this Transaction (if any) and what the capital gain of this sell is.
    ///
    /// Doing this modifies both Transactions. If the sell is less than this buy, this
    /// buy will have some amount left and the self returned Transaction will be Some.
    /// If the sell is greater than the lot, this Transaction will have nothing left
    /// and we will return None (while the second transaction will be Some).
    ///
    /// The calling code should throw out whichever Transaction is None as a result of
    /// this function.
    ///
    /// Returns the capital gain in terms of AUD.
    pub fn subtract_sell(&mut self, other: &mut Transaction) -> f64 {
        if !(self.transaction_type == TransactionType::Buy
            && other.transaction_type == TransactionType::Sell)
        {
            panic!(
                "Subtracting a sell from a buy is the only valid operation. Buy: {:?}, Sell: {:?}",
                self, other
            );
        }
        if self.currency != other.currency {
            panic!(
                "Tried to operate on transactions with different currencies. Buy: {:?}, Sell: {:?}",
                self, other
            );
        }
        let delta = self.amount.min(other.amount);
        debug!(
            "Subtracting {:?} from {:?}, decreasing both by {}",
            other, self, delta
        );
        let remaining_buy = self.amount - other.amount;
        let buy_in_aud = delta * self.rate;
        let sell_in_aud = delta * other.rate;
        if remaining_buy > 0.0 {
            self.amount -= other.amount;
            other.amount = 0.0;
        } else {
            other.amount -= self.amount;
            self.amount = 0.0;
        }
        let capital_gain = sell_in_aud - buy_in_aud;
        debug!("Buy is now {:?} and sell is now {:?}, capital gain is {}", self, other, capital_gain);
        capital_gain
    }
}
