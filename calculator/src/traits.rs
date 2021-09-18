use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use types::{Currency, Transaction};

pub trait Calculator {
    fn calculate_capital_gains(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<HashMap<Currency, f64>>;
}
