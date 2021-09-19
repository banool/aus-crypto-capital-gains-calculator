use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use types::{Currency, Transaction};

// Reexport things callers might to use.
pub use calculator::CalculatorType;
pub use reader::ReaderType;

#[derive(Debug)]
pub struct TransactionsFile {
    /// Path to the file.
    pub path: PathBuf,

    /// What reader to use for the transactions file.
    pub reader_type: ReaderType,
}

/// This function takes in a vec of TransactionsFiles, what readers to use for them,
/// and returns a vec of Transactions.
fn read_transactions(transactions_files: Vec<TransactionsFile>) -> Result<Vec<Transaction>> {
    let mut transactions: Vec<Transaction> = Vec::new();
    for transactions_file in transactions_files {
        let reader = transactions_file.reader_type.get_reader();
        let mut ts = reader.read_transactions(&transactions_file.path)?;
        transactions.append(&mut ts);
    }
    Ok(transactions)
}

/// This function takes in a vec of Transactions and processes them depending on the
/// chosen calcuator strategy.
pub fn calculate_capital_gains(
    transactions_files: Vec<TransactionsFile>,
    calculator_type: CalculatorType,
) -> Result<HashMap<Currency, f64>> {
    let transactions =
        read_transactions(transactions_files).context("Failed to read transactions")?;
    debug!("Transactions:");
    for t in &transactions {
        debug!("{:?}", t);
    }
    let calcuator = calculator_type.get_calculator();
    let capital_gains = calcuator
        .calculate_capital_gains(transactions)
        .context("Failed to calculate capital gains")?;
    Ok(capital_gains)
}
