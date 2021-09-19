use crate::traits::Reader;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use csv::Reader as CsvReader;
use log::trace;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use types::{Currency, Transaction, TransactionType};

fn comma_float<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    let buf = String::deserialize(deserializer)?;
    let buf = buf.replace(",", "");
    let num = buf.parse::<f64>();
    match num {
        Ok(num) => Ok(num),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
}

#[derive(Debug, Deserialize)]
struct Row {
    transacted_at: String,
    #[serde(deserialize_with = "comma_float")]
    debit: f64,
    currency: String,
    #[serde(deserialize_with = "comma_float")]
    counterparty_amount: f64,
    counterparty_currency: String,
    rates: String,
    #[serde(deserialize_with = "comma_float")]
    fee_amount: f64,
}

impl Into<Transaction> for Row {
    fn into(self) -> Transaction {
        let rate: String = self.rates.split(" = $").collect::<Vec<_>>()[1]
            .split_whitespace()
            .collect::<Vec<_>>()[0]
            .replace(",", "");
        let rate: f64 = rate
            .parse::<f64>()
            .expect(&format!("Failed to parse rate string {} as float", rate));
        // This ignores fees for now.
        let (currency, transaction_type) = match self.currency == "AUD".to_string() {
            true => (self.counterparty_currency, TransactionType::Buy),
            false => (self.currency, TransactionType::Sell),
        };
        let amount = match transaction_type {
            TransactionType::Buy => self.counterparty_amount,
            TransactionType::Sell => self.debit,
        };
        let ndt = NaiveDateTime::parse_from_str(&self.transacted_at, "%Y-%m-%d %H:%M:%S %Z")
            .expect("Failed to parse timestamp");
        let dt = DateTime::<Utc>::from_utc(ndt, Utc);
        let unixtime = dt.timestamp() as u64;
        Transaction::new(amount, Currency(currency), rate, transaction_type, unixtime)
    }
}

pub struct CoinjarReader {}

impl Reader for CoinjarReader {
    fn read_transactions(&self, path: &PathBuf) -> Result<Vec<Transaction>> {
        let mut rdr = CsvReader::from_path(&path)?;
        let mut rows = Vec::new();
        for result in rdr.deserialize() {
            let row: Row = match result {
                Ok(result) => result,
                Err(ref e) => {
                    trace!("Skipping row {:?}: {}", result, e);
                    continue;
                }
            };
            rows.push(row);
        }
        let transactions: Vec<Transaction> = rows.into_iter().map(|r| r.into()).collect();
        Ok(transactions)
    }
}
