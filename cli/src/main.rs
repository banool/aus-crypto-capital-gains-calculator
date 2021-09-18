use anyhow::Result;
use backend::{CalculatorType, ReaderType};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "aus-crypto-capital-gains-calculator",
    author = "Daniel Porteous"
)]
struct Args {
    /// Paths to files containing transactions.
    #[structopt(short, long)]
    paths: Vec<PathBuf>,

    /// Readers you want to use for these files.
    /// The order here must match the order of the given paths.
    #[structopt(short, long, required = true, possible_values = &ReaderType::variants(), case_insensitive = true)]
    readers: Vec<ReaderType>,

    /// Strategy you want to use for calculating the capital gains.
    #[structopt(short, long, required = true, possible_values = &CalculatorType::variants(), case_insensitive = true)]
    calculator: CalculatorType,
}

fn main() -> Result<()> {
    let args = Args::from_args();
    Ok(())
}
