use crate::constants::{EXTRACT_DIR, GITIGNORE_URL, MASTER_ZIP_PATH};
use anyhow::{Context, Result, bail};
use colored::Colorize;
use glob::{MatchOptions, glob_with};
use inquire::{Confirm, Select};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

pub async fn ensure_gitignore_master_downloaded_locally() -> Result<()> {
    let exists = tokio::fs::try_exists(MASTER_ZIP_PATH.clone())
        .await
        .context("failed to check if MASTER_ZIP_PATH exists")?;
    if exists {
        println!(
            "{}",
            format!(
                "File already exists at {:?}, skip downloading",
                MASTER_ZIP_PATH
                    .to_str()
                    .context("failed to convert MASTER_ZIP_PATH to str")?
            )
            .bright_black()
        );
        return Ok(());
    }
    println!(
        "{}",
        format!(
            "File does not exist at {:?}, downloading...",
            MASTER_ZIP_PATH
                .to_str()
                .context("failed to convert MASTER_ZIP_PATH to str")?
        )
        .bright_green()
    );
    let response = reqwest::get(GITIGNORE_URL)
        .await
        .context("failed to download GITIGNORE_URL")?;
    let bytes = response.bytes().await.context("failed to read zip file")?;
    tokio::fs::write(MASTER_ZIP_PATH.clone(), &bytes)
        .await
        .context("failed to write zip file")?;
    println!(
        "{}",
        format!("Finished downloading {:?}", MASTER_ZIP_PATH.to_str()).bright_green()
    );
    unzip_gitignore_master_side_by_side()
        .await
        .context("failed to unzip gitignore master")?;
    Ok(())
}

pub async fn unzip_gitignore_master_side_by_side() -> Result<()> {
    // Create the extraction directory if it doesn't exist
    tokio::fs::create_dir_all(EXTRACT_DIR.clone())
        .await
        .context("failed to create extraction directory")?;

    // Read the zip file into memory
    let zip_bytes = tokio::fs::read(MASTER_ZIP_PATH.clone())
        .await
        .context("failed to read zip file")?;
    let reader = std::io::Cursor::new(zip_bytes);

    // Open the zip archive
    let mut archive = ZipArchive::new(reader).context("failed to create zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = EXTRACT_DIR.join(file.name());

        if file.name().ends_with('/') {
            tokio::fs::create_dir_all(&outpath)
                .await
                .context("failed to create extraction sub-directory")?;
        } else {
            if let Some(parent) = outpath.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .context("failed to create extraction parent directory")?;
            }
            let mut outfile = File::create(&outpath)
                .await
                .context("failed to create outpath")?;
            let mut buffer = Vec::new();
            std::io::copy(&mut file, &mut buffer)?;
            outfile
                .write_all(&buffer)
                .await
                .context("failed to write to outfile")?;
        }
    }

    println!(
        "{}",
        format!(
            "Unzipped gitignore master to {:?}",
            EXTRACT_DIR
                .to_str()
                .context("failed to convert EXTRACT_DIR to str")?
        )
        .bright_green()
    );
    Ok(())
}

pub async fn list_gitignore_files_in_extract_dir(patt: Option<String>) -> Result<Vec<PathBuf>> {
    let mut gitignore_files = Vec::new();
    let mut glob_patt = PathBuf::from(EXTRACT_DIR.clone());
    glob_patt.push("**");
    let patt = patt.map_or("*.gitignore".to_string(), |f| format!("*{}*.gitignore", f));
    glob_patt.push(patt);
    let glob_patt = glob_patt.to_str().context("failed to build patt")?;
    println!(
        "{}",
        format!("Globbing file pattern: {}", glob_patt.bright_green())
    );
    for entry in glob_with(
        glob_patt,
        MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        },
    )
    .context("Failed to read glob pattern")?
    {
        match entry {
            Ok(path) => gitignore_files.push(path),
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(gitignore_files)
}

struct PathWrapper<'a> {
    path: &'a Path,
}

impl<'a> std::fmt::Display for PathWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

pub fn disambiguate_gitignore<'a>(paths: &'a Vec<&Path>) -> Result<&'a Path> {
    //see https://github.com/mikaelmello/inquire/blob/main/inquire/examples/multiselect.rs
    if paths.len() == 0 {
        bail!("no gitignore file match");
    } else if paths.len() == 1 {
        return Ok(paths[0]);
    }

    let prompt = "Pick a gitignore to use:".magenta().to_string();
    let ps = paths.iter().map(|p| PathWrapper { path: p }).collect();
    let picked = Select::new(&prompt, ps)
        .with_help_message(
            &"Select a branch to switch to (excluding current branch)"
                .bright_magenta()
                .to_string(),
        )
        .with_vim_mode(true)
        .prompt()
        .context("failed to pick a gitignore")?;
    return Ok(picked.path);
}

pub async fn use_ignore_template(gitignore_path: &PathBuf) -> Result<()> {
    let out_path = "./.gitignore";

    let mut yes = true;

    let exists = tokio::fs::try_exists(out_path)
        .await
        .context("failed to check .gitignore exists")?;
    if exists {
        yes = Confirm::new(".gitignore already exists. Are you sure to overrride?")
            .with_default(false)
            .prompt()
            .unwrap_or(false);
    }
    if yes {
        tokio::fs::copy(gitignore_path, "./.gitignore")
            .await
            .context("failed to read gitignore template")?;
        println!("{}", format!("wrote {}", out_path).on_bright_blue());
    } else {
        println!("{}", ".gitignore is not overridden".bright_black());
    }
    Ok(())
}
