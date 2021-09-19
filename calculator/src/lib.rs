use structopt::clap::arg_enum;

mod fifo;
mod traits;

use crate::fifo::FifoCalculator;
use crate::traits::Calculator;

arg_enum! {
/// This enum registers all the different calculator options.
#[derive(Debug)]
pub enum CalculatorType {
    Fifo,
}
}

impl CalculatorType {
    pub fn get_calculator(&self) -> Box<dyn Calculator> {
        match &self {
            Self::Fifo => Box::new(FifoCalculator {}),
        }
    }
}
