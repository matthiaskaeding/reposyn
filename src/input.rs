use crate::summary::input_summary;
use git2::Repository;
use globset::{Glob, GlobSetBuilder};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use time::OffsetDateTime;

const TEXT_EXTENSIONS: &[&str] = &[
    "bash", "conf", "css", "csv", "htm", "html", "js", "json", "md", "py", "rs", "sh", "toml",
    "txt", "xml", "yaml", "yml",
];
pub fn input_context(input_dir: &str, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let repo_name = if input_dir == "./" {
        let path = Path::new(".")
            .canonicalize()?
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(input_dir)
            .to_string(); // Convert to owned String
        path
    } else {
        input_dir.to_string() // Convert to owned String
    };

    writeln!(
        output,
        "<context>
You are an expert programming Al assistant who receives a summary of repo {} in XML format.
Understand the contents of the repo.
</context>\n\n",
        repo_name
    )?;

    Ok(())
}

pub fn input_repo_stats(repo: &Repository, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut count = 0;
    let mut first_time = None;
    let mut latest_time = None;
    let mut recent_messages = Vec::new();

    for commit_id in revwalk {
        let commit = repo.find_commit(commit_id?)?;
        let time = commit.time().seconds();

        if count == 0 {
            latest_time = Some(time);
        }
        if count < 3 {
            if let Some(msg) = commit.message() {
                recent_messages.push(msg.to_string());
            }
        }

        first_time = Some(time);
        count += 1;
    }

    writeln!(output, "<Repo statistics>")?;
    if let (Some(first), Some(latest)) = (first_time, latest_time) {
        let first_date = OffsetDateTime::from_unix_timestamp(first)?;
        let latest_date = OffsetDateTime::from_unix_timestamp(latest)?;
        writeln!(output, "<Total commits>{}</Total commits>", count)?;
        writeln!(output, "<First commit>{}</First commit>", first_date)?;
        writeln!(output, "<Latest commits>{}</Latest commits>", latest_date)?;
    }
    writeln!(output, "</Repo statistics>")?;

    writeln!(output, "<Last three commit messages>")?;
    for (i, msg) in recent_messages.iter().enumerate() {
        let msg_clean: String = msg
            .lines()
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("\n");
        writeln!(
            output,
            "<Commit_message_{}>{}</Commit_message_{}>",
            i, msg_clean, i,
        )?;
    }
    writeln!(output, "</Last three commit messages>\n\n")?;
    Ok(())
}

// This is workhorse which loops over all files in the repo
pub fn input_files(
    repo_dir: &str,
    output: &mut impl Write,
    ignore_patterns: Vec<&str>,
    summarize_glob_patterns: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let mut sizes = BTreeSet::new();

    let mut override_builder = OverrideBuilder::new(repo_dir);
    for pattern in &ignore_patterns {
        override_builder.add(&format!("!{}", pattern))?;
    }
    let overrides = override_builder.build()?;

    let walker = WalkBuilder::new(repo_dir)
        .hidden(true)
        .overrides(overrides)
        .git_ignore(true)
        .build();
    let mut paths = Vec::new();

    // Use Gitignore to match
    let mut glob_builder = GlobSetBuilder::new();
    for pattern in summarize_glob_patterns.iter() {
        glob_builder.add(Glob::new(pattern)?);
    }
    let glob_set = glob_builder.build()?;

    let mut n_files = 0;
    'file_loop: for entry in walker {
        let path = match entry {
            Ok(entry) => entry.path().to_path_buf(),
            Err(err) => {
                println!("ERROR: {}", err);
                continue;
            }
        };

        if path.is_file() {
            n_files += 1;
            if let Some(extension) = path.extension() {
                // Skip non-text file by simply looking at extension
                let extension = extension.to_string_lossy().to_lowercase();
                if !TEXT_EXTENSIONS.contains(&extension.as_str()) {
                    continue;
                }
                let path_string = path.display().to_string();
                if glob_set.is_match(path_string.to_string()) {
                    //println!("Summarazing this file: {}", path_string);
                    match input_summary(&path, output) {
                        Ok(()) => (),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    continue 'file_loop;
                }

                // Write contents into output
                let mut content = String::new();
                let mut file = File::open(&path)?;
                file.read_to_string(&mut content)?;
                writeln!(
                    output,
                    "<File:{}>\n{}</File:{}>\n",
                    path_string, content, path_string
                )?;

                paths.push(path_string.clone());
                // Store size
                let metadata = path.metadata()?;
                let size_pair = (metadata.len(), path_string.clone());
                if sizes.len() < 5 {
                    sizes.insert(size_pair);
                } else if let Some(smallest) = sizes.first().cloned() {
                    if size_pair > smallest {
                        sizes.remove(&smallest); // Remove the smallest element
                        sizes.insert(size_pair);
                    }
                }
            }
        }
    }

    // Input paths
    writeln!(output, "<All paths>")?;
    for p in paths.iter() {
        writeln!(output, "{}", p)?;
    }
    writeln!(output, "</All paths>")?;
    if n_files > 0 {
        println!(
            "Biggest files: path (size). Showing {} of {}",
            std::cmp::min(5, sizes.len()),
            n_files,
        );
        let mut count = 1;
        for item in sizes.iter().rev() {
            println!("{}: {} ({})", count, item.1, format_size(item.0));
            count += 1;
        }
    } else {
        println!("No files found");
    }

    Ok(())
}

// Utils

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.2} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
