use crate::traits::Reader;
use anyhow::Result;
use std::path::PathBuf;
use types::Transaction;

pub struct CoinjarSimpleReader {}

impl Reader for CoinjarSimpleReader {
    fn read_transactions(&self, _path: &PathBuf) -> Result<Vec<Transaction>> {
        todo!();
    }
}
