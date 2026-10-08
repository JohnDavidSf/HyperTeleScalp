use std::{env, fs::File, io::BufReader, path::Path, process::ExitCode};

use hypertele_scalp::{config::Config, parser::SyntheticFixtureParser, replay::replay};
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)]
    Config(#[from] hypertele_scalp::config::ConfigError),
    #[error(transparent)]
    Replay(#[from] hypertele_scalp::replay::ReplayError),
    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("usage: hypertele-scalp check-config <path> | replay-fixtures <normalized.ndjson>")]
    Usage,
}

fn run() -> Result<(), AppError> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or(AppError::Usage)?;
    let path = args.next().ok_or(AppError::Usage)?;
    if args.next().is_some() {
        return Err(AppError::Usage);
    }
    match command.as_str() {
        "check-config" => {
            let config = Config::load(Path::new(&path))?;
            println!(
                "configuration valid; mode={:?}; LIVE TRADING DISABLED",
                config.backend
            );
        }
        "replay-fixtures" => {
            let summary = replay(BufReader::new(File::open(path)?), &SyntheticFixtureParser)?;
            println!("offline fixture classification: {summary:?}; no orders submitted");
        }
        _ => return Err(AppError::Usage),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
