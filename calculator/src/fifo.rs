use crate::Calculator;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use types::{Currency, Transaction};

pub struct FifoCalculator {}

impl Calculator for FifoCalculator {
    fn calculate_capital_gains(
        &self,
        mut transactions: Vec<Transaction>,
    ) -> Result<HashMap<Currency, f64>> {
        // Sort transactions by unixtime.
        transactions.sort_by(|a, b| a.unixtime.partial_cmp(&b.unixtime).unwrap());
        Ok(HashMap::new())
    }
}
