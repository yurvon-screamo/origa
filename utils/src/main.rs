mod api;
mod cli;
mod commands;
mod dictionary;
mod utils;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::commands::{run_find_missing, run_ndlocr, run_tokenize, run_tokenize_well_known};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Tokenize { text, file } => run_tokenize(text, file),
        Commands::Ndlocr {
            input,
            detector,
            rec30,
            rec50,
            rec100,
            vocab,
        } => run_ndlocr(input, detector, rec30, rec50, rec100, vocab),
        Commands::TokenizeWellKnown { path } => run_tokenize_well_known(path),
        Commands::FindMissing {
            output,
            generate,
            api_base,
            api_key,
            workers,
            chunk_size,
            russian_only,
            english_only,
        } => {
            run_find_missing(
                output,
                generate,
                api_base,
                api_key,
                workers,
                chunk_size,
                russian_only,
                english_only,
            )
            .await
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
