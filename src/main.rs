use crate::glob::gitignore_to_glob;

use std::collections::HashSet;
mod config;
mod file_merge;
mod glob;
mod input;
mod summary;
use clap::{Arg, Command};
pub use config::RepoConfig;

#[cfg(test)]
pub(crate) mod test_utils;

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
        .arg(
            Arg::new("thr_summary")
                .short('t')
                .long("thr_summary")
                .help("Files larger than this threshold (kb) will be summarized")
                .default_value(
                    "64",
                ),
            )
        .get_matches();

    let created_at = std::time::Instant::now();
    let input_folder = matches
        .get_one::<String>("input_folder")
        .unwrap()
        .to_string();
    let repo_path = std::path::PathBuf::from(input_folder);
    let ignore = matches.get_one::<String>("ignore").unwrap();
    let summarize = matches.get_one::<String>("summarize").unwrap();
    let output_file = matches
        .get_one::<String>("output_file")
        .unwrap()
        .to_string();
    let use_clipboard = matches.get_flag("clipboard");
    let extensions = matches.get_one::<String>("extensions_text").unwrap();
    let thr = matches
        .get_one::<String>("thr_summary")
        .unwrap()
        .parse::<f64>()
        .unwrap();

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
                Some(glob_pattern) => summarize_patterns.push(glob_pattern),
                None => panic!("Invalid gitignore pattern: '{}'", pattern),
            }
        }
    }

    let config = RepoConfig::new(
        repo_path,
        use_clipboard,
        output_file,
        ignore_patterns,
        summarize_patterns,
        text_extensions,
        created_at,
        thr,
    )?;

    file_merge::merge_files(&config)?;
    Ok(())
}
