use crate::glob::gitignore_to_glob;

use git2::Repository;
use std::collections::HashSet;
use std::path::Path;
mod config;
mod file_merge;
mod glob;
mod input;
mod summary;
use clap::{Arg, Command};
pub use config::RepoConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("reposyn")
        .version("0.1.0")
        .author("Matthias Kaeding <kaedingmatthias@gmail.com>")
        .about("Creates AI-friendly text summary of a repo")
        .arg(
            Arg::new("input_folder")
                .short('f')
                .long("folder")
                .value_name("DIR")
                .help("Folder to summarize")
                .default_value("./"),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("PATTERNS")
                .help("Comma-separated paths to ignore (e.g., 'target,node_modules')")
                .default_value("repo-synopsis.txt"),
        )
        .arg(
            Arg::new("summarize")
                .short('s')
                .long("summarize")
                .value_name("SUMMARIZE")
                .help("Comma-separated paths to summarise (e.g., '*.json' for all json files)")
                .default_value(""),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output_file")
                .value_name("output_file")
                .help("Output text file")
                .default_value("repo-synopsis.txt"),
        )
        .arg(
            Arg::new("clipboard")
                .short('c')
                .long("clipboard")
                .help("Copy to clipboard, will ignore <output_file> if set")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("extensions_text")
                .short('e')
                .long("extensions")
                .help("Comma seperated string of extensions marking text files - other files will not be included")
                .default_value(
                    "bash,conf,css,csv,htm,html,js,json,md,py,rs,sh,toml,txt,xml,yaml,yml",
                ),
        )
        .get_matches();

    let repo_dir = matches
        .get_one::<String>("input_folder")
        .unwrap()
        .to_string();
    let ignore = matches.get_one::<String>("ignore").unwrap();
    let summarize = matches.get_one::<String>("summarize").unwrap();
    let output_file = matches
        .get_one::<String>("output_file")
        .unwrap()
        .to_string();
    let use_clipboard = matches.get_flag("clipboard");
    let extensions = matches.get_one::<String>("extensions_text").unwrap();

    // Build the values for the RepoConfig from the arguments
    let repo = match Repository::open(&repo_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let ignore_patterns: Vec<String> = if ignore.is_empty() {
        Vec::new()
    } else {
        ignore.split(",").map(String::from).collect()
    };
    let text_extensions: HashSet<_> = extensions
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();

    let mut summarize_patterns: Vec<String> = Vec::new();
    if !summarize.is_empty() {
        let summarize_split = summarize.split(",");
        for pattern in summarize_split {
            let res = gitignore_to_glob(pattern);
            match res {
                Some(glob_pattern) => {
                    // We'll need to ensure the glob_pattern has a lifetime that matches the Vec
                    // This might require changes to gitignore_to_glob's return type
                    // or storing the patterns differently depending on your use case
                    summarize_patterns.push(glob_pattern)
                }
                None => panic!("Invalid gitignore pattern: '{}'", pattern),
            }
        }
    }

    let repo_name = if repo_dir == "./" {
        let path = Path::new(".")
            .canonicalize()?
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&repo_dir)
            .to_string(); // Convert to owned String
        path
    } else {
        repo_dir.to_string()
    };
    let config = RepoConfig {
        repo_dir,
        repo_name,
        repo,
        use_clipboard,
        output_file,
        ignore_patterns,
        summarize_patterns,
        text_extensions,
    };

    file_merge::merge_files(&config)?;
    Ok(())
}
