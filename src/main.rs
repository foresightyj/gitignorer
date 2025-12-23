#[macro_use]
extern crate lazy_static;

use std::path::{Path, StripPrefixError};

use crate::constants::EXTRACT_DIR;
use colored::Colorize;
mod constants;
mod utils;
use anyhow::{Context, Result};

use clap::{Parser, Subcommand};
use utils::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Adds files to myapp
    Create {
        lang: Option<String>,
    },
    UpdateCache,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::Create { lang } => {
            let gitignore_paths = list_gitignore_files_in_extract_dir(lang.clone())
                .await
                .context("failed to list ignore files")?;
            let mut base_dir = EXTRACT_DIR.clone();
            base_dir.push("gitignore-main");
            let relative_paths: Result<Vec<&Path>, StripPrefixError> = gitignore_paths
                .iter()
                .map(|f| f.strip_prefix(&base_dir))
                .collect();
            let relative_paths = relative_paths.context("failed to strip path")?;
            let picked = disambiguate_gitignore(&relative_paths)
                .context("failed to match or pick a gitignore template to use")?;
            let full_path = base_dir.join(picked);
            use_ignore_template(&full_path)
                .await
                .context("failed to pick a gitignore template")?;
        }
        Commands::UpdateCache => {
            ensure_gitignore_master_downloaded_locally()
                .await
                .context("failed to download the gitignore zip file")?;
            println!("{}", "Updated gitignore cache".bright_blue());
        }
    }

    Ok(())
}
