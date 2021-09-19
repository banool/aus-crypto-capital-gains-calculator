use anyhow::{bail, Result};
use backend::{calculate_capital_gains, CalculatorType, ReaderType, TransactionsFile};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use structopt::clap::AppSettings::ColoredHelp;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "aus-crypto-capital-gains-calculator",
    author = "Daniel Porteous",
    setting(ColoredHelp),
)]
struct Args {
    /// Whether to enable debug logging.
    #[structopt(short, long)]
    debug: bool,

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

    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    if args.paths.len() != args.readers.len() {
        bail!("Please pass 1 reader per path");
    }
    let mut transactions_files = Vec::new();
    for (i, path) in args.paths.iter().enumerate() {
        let reader = &args.readers[i];
        let transactions_file = TransactionsFile {
            path: path.clone(),
            reader_type: reader.clone(),
        };
        transactions_files.push(transactions_file);
    }
    let capital_gains = calculate_capital_gains(transactions_files, args.calculator)?;
    let mut capital_gains: Vec<_> = capital_gains.into_iter().collect();
    capital_gains.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
    for (currency, capital_gain) in capital_gains {
        info!("Capital gain for {}: ${:.2} AUD", currency.0, capital_gain);
    }
    Ok(())
}
