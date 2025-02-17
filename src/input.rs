use crate::config::RepoConfig;
use crate::summary::input_summary;
use globset::{Glob, GlobSetBuilder};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use time::OffsetDateTime;

pub fn input_context(repo_name: &str, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let prompt = format!(
        r#"1. You are an expert software engineer analyzing code repositories
2. You received a text summary of git repo ({})
3. Think about the contents and understand the purpose of the code.
"#,
        repo_name
    );
    writeln!(output, "\n<context>\n{}</context>\n\n", prompt)?;

    Ok(())
}

pub fn input_repo_stats(
    config: &RepoConfig,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let repo = config.open_repo().expect("failed to open repo");

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let n_commits = revwalk.count();

    revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut latest_time = None;

    let mut count = 0;

    let mut buffer = String::with_capacity(100);

    buffer.push_str("<Repo statistics>\n");
    let line = format!("<Number of commits>{}</Number of commits>\n", n_commits);
    buffer.push_str(&line);
    buffer.push_str("<Recent commits, most recent first>\n");

    for commit_id in revwalk {
        let commit = repo.find_commit(commit_id?)?;
        if count == 0 {
            let time = commit.time().seconds();
            latest_time = Some(time);
        }
        if let Some(msg) = commit.message() {
            let msg_clean: String = msg
                .lines()
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join("\n");

            let line = format!("<commit_{}>{}\n", count, msg_clean);
            buffer.push_str(&line); // Push the owned String, not a reference
        }

        count += 1;
        if count == 3 {
            break;
        }
    }
    buffer.push_str("</Recent commits, most recent first>\n");

    if let Some(latest) = latest_time {
        let timestamp = OffsetDateTime::from_unix_timestamp(latest)?;
        let msg_first = format!("<Latest commit>{}</Latest commit>\n", timestamp);
        buffer.push_str(&msg_first);
    }

    revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let first_commit = revwalk.last();
    if let Some(first_commit_id) = first_commit {
        let commit = repo.find_commit(first_commit_id?)?;
        let time = commit.time().seconds();
        let first_time = Some(time);

        if let Some(first) = first_time {
            let timestamp = OffsetDateTime::from_unix_timestamp(first)?;
            let msg_first = format!("<First commit>{}</First commit>\n", timestamp);
            buffer.push_str(&msg_first);
        }
    }

    buffer.push_str("</Repo statistics>\n");

    output.write_all(buffer.as_bytes())?;
    Ok(())
}

const BUFFER_THRESHOLD: usize = 10 * 1024 * 1024; // 10MB

// Main workhorse which loops over all files in the repo
pub fn input_files(config: &RepoConfig, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let mut sizes = BTreeSet::new();

    let mut override_builder = OverrideBuilder::new(&config.repo_path);
    for pattern in config.ignore_patterns.iter() {
        override_builder.add(&format!("!{}", pattern))?;
    }
    let overrides = override_builder.build()?;

    let walker = WalkBuilder::new(&config.repo_path)
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
    let mut lines = String::with_capacity(100);

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

        let metadata = path.metadata()?;
        let size_in_kb = metadata.len() as f64 / 1024.0;
        if glob_set.is_match(&path_string) || size_in_kb > config.thr {
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

        let line = format!(
            "<File:{}>\n{}</File:{}>\n",
            path_string, content, path_string
        );
        lines.push_str(&line);
        // Reset if buffer
        if lines.len() > BUFFER_THRESHOLD {
            output.write_all(lines.as_bytes())?;
            lines.clear();
        }
        paths.push(path_string.clone());

        // Store size
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
    lines.push_str("<All paths>\n");
    for p in paths.iter() {
        lines.push_str(p);
    }
    lines.push_str("</All paths>\n");

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

    output.write_all(lines.as_bytes())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_repo;
    use std::fs;

    #[test]
    fn test_input_context() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        input_context("test-repo", &mut output)?;

        let result = String::from_utf8(output)?;
        assert!(result.contains("You are an expert software engineer"));
        assert!(result.contains("test-repo"));
        assert!(result.starts_with("<context>"));
        assert!(result.contains("</context>"));

        Ok(())
    }

    #[test]
    fn test_input_repo_stats() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, config) = setup_test_repo()?;
        let mut output = Vec::new();
        input_repo_stats(&config, &mut output)?;
        let result = String::from_utf8(output)?;
        println!("{}", result);

        // Check all required tags and content
        assert!(result.contains("<Repo statistics>"));
        assert!(result.contains("</Repo statistics>"));
        assert!(result.contains("<Number of commits>1</Number of commits>"));
        assert!(result.contains("<Recent commits, most recent first>"));
        assert!(result.contains("</Recent commits, most recent first>"));

        // Check commit message format
        assert!(result.contains("<commit_0>Initial commit"));

        // Check timestamp tags
        assert!(result.contains("<First commit>"));
        assert!(result.contains("</First commit>"));
        assert!(result.contains("<Latest commit>"));
        assert!(result.contains("</Latest commit>"));

        Ok(())
    }

    #[test]
    fn test_input_files() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, config) = setup_test_repo()?;

        // Create additional test files
        fs::write(
            config.repo_path.join("test1.txt"),
            "First test file content",
        )?;
        fs::write(
            config.repo_path.join("test2.txt"),
            "Second test file content",
        )?;
        fs::write(config.repo_path.join("test.json"), r#"{"key": "value"}"#)?;

        let mut output = Vec::new();
        input_files(&config, &mut output)?;

        let result = String::from_utf8(output)?;

        // Check if files are included
        assert!(result.contains("<File:"));
        assert!(result.contains("test1.txt"));
        assert!(result.contains("test2.txt"));
        assert!(result.contains("First test file content"));
        assert!(result.contains("Second test file content"));

        // Check if paths section exists
        assert!(result.contains("<All paths>"));
        assert!(result.contains("</All paths>"));

        // Test file extension filtering
        assert!(!result.contains("test.json")); // JSON files should be excluded by default

        Ok(())
    }

    #[test]
    fn test_input_files_with_ignore_patterns() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, mut config) = setup_test_repo()?;

        // Add ignore pattern
        config.ignore_patterns = vec!["test1.txt".to_string()];

        // Create test files
        fs::write(config.repo_path.join("test1.txt"), "Should be ignored")?;
        fs::write(config.repo_path.join("test2.txt"), "Should be included")?;

        let mut output = Vec::new();
        input_files(&config, &mut output)?;

        let result = String::from_utf8(output)?;
        assert!(!result.contains("Should be ignored"));
        assert!(result.contains("Should be included"));

        Ok(())
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_input_files_with_summarize_patterns() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, mut config) = setup_test_repo()?;

        // Add summarize pattern for json files
        config.summarize_patterns = vec!["**/*.json".to_string()];
        config.text_extensions.insert("json".to_string());

        // Create test files
        fs::write(config.repo_path.join("test.txt"), "Regular text file")?;
        fs::write(
            config.repo_path.join("test.json"),
            r#"{"key": "value", "array": [1,2,3]}"#,
        )?;

        let mut output = Vec::new();
        input_files(&config, &mut output)?;

        let result = String::from_utf8(output)?;
        assert!(result.contains("<Summary of file:"));
        assert!(result.contains("test.json"));
        assert!(result.contains("<File:"));
        assert!(result.contains("test.txt"));

        Ok(())
    }
}
