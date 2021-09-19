use structopt::clap::arg_enum;

mod coinjar;
mod coinjar_simple;
mod traits;

use crate::coinjar::CoinjarReader;
use crate::coinjar_simple::CoinjarSimpleReader;
use crate::traits::Reader;

arg_enum! {
/// This enum registers all the different reader options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReaderType {
    Coinjar,
    CoinjarSimple,
}
}

impl ReaderType {
    pub fn get_reader(&self) -> Box<dyn Reader> {
        match &self {
            Self::Coinjar => Box::new(CoinjarReader {}),
            Self::CoinjarSimple => Box::new(CoinjarSimpleReader {}),
        }
    }
}
