use anyhow::Result;
use chrono::TimeZone;
use std::path::PathBuf;
use types::Transaction;

pub trait Reader {
    fn read_transactions(&self, path: &PathBuf) -> Result<Vec<Transaction>>;
}
