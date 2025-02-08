use crate::config::RepoConfig;
use crate::summary::input_summary;
use git2::Repository;
use globset::{Glob, GlobSetBuilder};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use time::OffsetDateTime;

pub fn input_context(repo_name: &str, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let prompt = format!(
        r#"You are an expert software engineer who receives a summary of repo called {}.
    Analyze the repo, understand the problem the repo is solving."#,
        repo_name
    );
    writeln!(output, "<context>{}</context>\n\n", prompt)?;

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

// Main workhorse which loops over all files in the repo
pub fn input_files(config: &RepoConfig, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let mut sizes = BTreeSet::new();

    let mut override_builder = OverrideBuilder::new(&config.repo_dir);
    for pattern in config.ignore_patterns.iter() {
        override_builder.add(&format!("!{}", pattern))?;
    }
    let overrides = override_builder.build()?;

    let walker = WalkBuilder::new(&config.repo_dir)
        .hidden(true)
        .overrides(overrides)
        .git_ignore(true)
        .build();
    let mut paths = Vec::new();
    let mut glob_builder = GlobSetBuilder::new();
    for pattern in config.summarize_patterns.iter() {
        glob_builder.add(Glob::new(pattern)?);
    }
    let glob_set = glob_builder.build()?;

    let mut n_files = 0;
    for entry in walker {
        let path = match entry {
            Ok(entry) => entry.path().to_path_buf(),
            Err(err) => {
                println!("ERROR: {}", err);
                continue;
            }
        };
        if !path.is_file() {
            continue;
        }
        n_files += 1;
        // Skip non-text file by looking at extension
        if let Some(extension) = path.extension() {
            let extension = extension.to_string_lossy().to_lowercase();
            if !config.text_extensions.contains(&extension) {
                continue;
            }
        }

        let path_string = path.display().to_string();
        if glob_set.is_match(&path_string) {
            match input_summary(&path, output) {
                Ok(()) => (),
                Err(e) => eprintln!("Error: {}", e),
            }
            continue;
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
    if n_files == 0 {
        println!("No files found");
        return Ok(());
    }

    // Input paths
    writeln!(output, "<All paths>")?;
    for p in paths.iter() {
        writeln!(output, "{}", p)?;
    }
    writeln!(output, "</All paths>")?;

    println!(
        "Biggest files completely written to output: path (size). Showing {} of {}",
        std::cmp::min(5, sizes.len()),
        n_files,
    );
    let mut count = 1;
    for item in sizes.iter().rev() {
        println!("{}: {} ({})", count, item.1, format_size(item.0));
        count += 1;
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
