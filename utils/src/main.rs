mod api;
mod cli;
mod commands;
mod dictionary;
mod signing;
mod utils;

use clap::Parser;
use origa::domain::OrigaError;

use crate::cli::{CdnCommands, Cli, Commands};
use crate::commands::{
    run_build_phrase_dataset, run_find_missing, run_generate_grammar, run_generate_grammar_prompt,
    run_list, run_migrate_phrase_dataset, run_ndlocr, run_tokenize, run_tokenize_well_known,
    run_upload, run_upload_audio, run_validate_dictionary,
};

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
        Commands::BuildPhraseDataset {
            input,
            output,
            min_tokens,
        } => run_build_phrase_dataset(input, output, min_tokens),
        Commands::MigratePhraseDataset { dataset } => run_migrate_phrase_dataset(dataset),
        Commands::Cdn { command } => {
            let result: Result<(), Box<dyn std::error::Error>> = match command {
                CdnCommands::Upload { dir } => run_upload(dir).await,
                CdnCommands::UploadAudio {
                    dir,
                    workers,
                    only_failed,
                } => run_upload_audio(dir, workers, only_failed).await,
                CdnCommands::List { prefix } => run_list(prefix).await,
            };
            result.map_err(|e| OrigaError::TokenizerError {
                reason: e.to_string(),
            })
        },
        Commands::ValidateDictionary {
            api_key,
            api_base,
            model,
            workers,
            output,
            dry_run,
            limit,
        } => {
            run_validate_dictionary(api_key, api_base, model, workers, output, dry_run, limit).await
        },
        Commands::GenerateGrammarPrompt {
            title,
            level,
            language,
            rule_name_from_index,
        } => run_generate_grammar_prompt(title, level, language, rule_name_from_index),
        Commands::GenerateGrammar {
            rule_id,
            all,
            level,
            api_base,
            api_key,
            workers,
            dry_run,
            grammar_path,
        } => {
            run_generate_grammar(
                rule_id,
                all,
                level,
                api_base,
                api_key,
                workers,
                dry_run,
                grammar_path,
            )
            .await
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
