use crate::traits::Reader;
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use types::Transaction;

pub struct CoinjarSimpleReader {}

impl Reader for CoinjarSimpleReader {
    fn read_transactions(&self, path: &PathBuf) -> Result<Vec<Transaction>> {
        todo!();
        Ok(Vec::new())
    }
}
