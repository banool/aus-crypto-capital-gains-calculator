use crate::traits::Reader;
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use types::Transaction;

pub struct CoinjarReader {}

impl Reader for CoinjarReader {
    fn read_transactions(&self, path: &PathBuf) -> Result<Vec<Transaction>> {
        Ok(Vec::new())
    }
}
