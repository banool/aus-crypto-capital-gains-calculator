use anyhow::Result;
use std::path::PathBuf;
use types::Transaction;

pub trait Reader {
    fn read_transactions(&self, path: &PathBuf) -> Result<Vec<Transaction>>;
}
