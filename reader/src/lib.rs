use structopt::clap::arg_enum;
use types::{Transaction, TransactionType};

mod coinjar;
mod mystery;
mod traits;

use crate::coinjar::CoinjarReader;
use crate::mystery::MysteryReader;
use crate::traits::Reader;

arg_enum! {
/// This enum registers all the different reader options.
#[derive(Debug)]
pub enum ReaderType {
    Coinjar,
    Mystery,
}
}

impl ReaderType {
    pub fn get_reader(&self) -> Box<dyn Reader> {
        match &self {
            Self::Coinjar => Box::new(coinjar::CoinjarReader {}),
            Self::Mystery => Box::new(mystery::MysteryReader {}),
        }
    }
}
