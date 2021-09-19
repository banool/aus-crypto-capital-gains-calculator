use structopt::clap::arg_enum;
use types::{Transaction, TransactionType};

mod coinjar;
mod coinjar_simple;
mod traits;

use crate::coinjar::CoinjarReader;
use crate::coinjar_simple::CoinjarSimpleReader;
use crate::traits::Reader;

arg_enum! {
/// This enum registers all the different reader options.
#[derive(Clone, Debug)]
pub enum ReaderType {
    Coinjar,
    CoinjarSimple,
}
}

impl ReaderType {
    pub fn get_reader(&self) -> Box<dyn Reader> {
        match &self {
            Self::Coinjar => Box::new(coinjar::CoinjarReader {}),
            Self::CoinjarSimple => Box::new(coinjar_simple::CoinjarSimpleReader {}),
        }
    }
}
