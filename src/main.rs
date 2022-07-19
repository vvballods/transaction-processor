#[macro_use]
extern crate serde_derive;

use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::Stdout;

use anyhow::Ok;
use csv::{Reader, ReaderBuilder, Trim, Writer};
use domain::Transaction;
use processor::TransactionProcessor;
use state::State;
use structopt::StructOpt;

mod api;
mod domain;
mod processor;
mod state;

#[derive(Debug, StructOpt)]
pub struct Config {
    #[structopt(parse(from_os_str))]
    pub path: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    init_logging();
    tracing::info!("Starting transactions processor...");
    let config = Config::from_args();
    let transactions_path = current_dir()?.join(config.path);
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(transactions_path)?;
    let mut writer = Writer::from_writer(io::stdout());
    process(&mut reader, &mut writer)?;
    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();
}

fn process(reader: &mut Reader<File>, writer: &mut Writer<Stdout>) -> Result<(), anyhow::Error> {
    let processor = TransactionProcessor::new(State::new());

    for record in reader.deserialize::<Transaction>() {
        let _ = record
            .map(|transaction| processor.process(transaction.into()))
            .map_err(anyhow::Error::from);
    }

    let mut balances = processor.get_accounts()?.into_iter();
    while let Some(mut balance) = balances.next() {
        balance.scaled();
        writer.serialize(balance)?;
    }

    writer.flush()?;

    Ok(())
}
