use crate::Calculator;
use anyhow::{bail, Result};
use log::debug;
use std::collections::{HashMap, VecDeque};
use types::{Currency, Transaction, TransactionType};

pub struct FifoCalculator {}

impl Calculator for FifoCalculator {
    fn calculate_capital_gains(
        &self,
        mut transactions: Vec<Transaction>,
    ) -> Result<HashMap<Currency, f64>> {
        // Sort transactions by unixtime.
        transactions.sort_by(|a, b| a.unixtime.partial_cmp(&b.unixtime).unwrap());

        // Figure out all the currencies we're working with.
        let currencies: Vec<Currency> = transactions.iter().map(|t| t.currency.clone()).collect();

        // Get the capital gain for each currency.
        let mut capital_gains = HashMap::new();
        for currency in currencies.into_iter() {
            debug!("Determining capital gain for {}", currency.0);
            let mut currency_transactions: Vec<Transaction> = Vec::new();
            for t in transactions.iter() {
                if t.currency == currency {
                    currency_transactions.push(t.clone());
                }
            }
            let capital_gain = self.calculate_capital_gains_single_currency(currency_transactions)?;
            capital_gains.insert(currency, capital_gain);
        }

        Ok(capital_gains)
    }
}

impl FifoCalculator {
    // This function does not assert that all the transactions are indeed
    // for a single currency, but if they're not, it'll fail down the line.
    fn calculate_capital_gains_single_currency(&self, transactions: Vec<Transaction>) -> Result<f64> {
        // We keep track of purchases as individual lots in a queue (FIFO).
        let mut lots: VecDeque<Transaction> = VecDeque::new();

        // Track the ultimate capital gain.
        let mut capital_gain = 0.0;

        for transaction in transactions.into_iter() {
            match transaction.transaction_type {
                TransactionType::Buy => lots.push_back(transaction),
                TransactionType::Sell => {
                    // While this transaction has value remaining, use it to
                    // to subtract value from the lot at the head of the queue.
                    let mut sell = transaction;  // Clearer naming for what is happening.
                    while !sell.is_exhausted() {
                        if lots.len() == 0 {
                            // If we get to this condition, it means that while trying
                            // to figure out the capital gain for this sell, we ran out
                            // of buys against which to apply this sell. This invariably
                            // means we have incomplete data, specifically that we are
                            // missing sell events.
                            bail!("There is a sell Transaction but no buy remaining to subtract it from {:?}. This means the data is incomplete, and specifically is missing buy events", sell);
                        }
                        let capital_gain_delta = lots[0].subtract_sell(&mut sell);
                        if lots[0].is_exhausted() {
                            // There is nothing left in this buy, pop it and move to
                            // next one to continue to deplete this sell.
                            lots.pop_front();
                        }
                        capital_gain += capital_gain_delta;
                    }
                    // There is nothing left in this sell, move on to the next transaction.
                }
            }
        }

        Ok(capital_gain)
    }
}
