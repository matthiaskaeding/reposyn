use crate::glob::gitignore_to_glob;
use copypasta::{ClipboardContext, ClipboardProvider};
use git2::Repository;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use regex::Regex;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "json", "yaml", "yml", "toml", "css", "html", "htm", "xml",
    "csv", "sh", "bash", "conf",
];

fn input_context(input_dir: &str, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
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

fn input_repo_stats(repo: &Repository, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
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

fn format_size(size: u64) -> String {
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

fn summarize_text_file(filepath: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    // Read file contents and get file size
    let mut file = File::open(filepath)?;
    let metadata = std::fs::metadata(filepath)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0); // Convert bytes to MB

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Get first and last 3 lines
    let reader = BufReader::new(File::open(filepath)?);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let first_three = lines.iter().take(3).cloned().collect::<Vec<_>>();
    let last_three = lines
        .iter()
        .rev()
        .take(3)
        .rev()
        .cloned()
        .collect::<Vec<_>>();

    // Compile regex patterns
    let email_regex = Regex::new(r"[\w\.-]+@[\w\.-]+\.\w+")?;
    let url_regex = Regex::new(
        r"http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+",
    )?;
    let date_regex = Regex::new(r"\d{1,2}[-/]\d{1,2}[-/]\d{2,4}")?;
    let special_char_regex = Regex::new(r"[^a-zA-Z0-9\s]")?;

    // Count patterns
    let email_count = email_regex.find_iter(&contents).count();
    let url_count = url_regex.find_iter(&contents).count();
    let date_count = date_regex.find_iter(&contents).count();

    // Count special characters
    let mut special_chars: HashMap<String, usize> = HashMap::new();
    for special_char in special_char_regex.find_iter(&contents) {
        let char_str = special_char.as_str().to_string();
        *special_chars.entry(char_str).or_insert(0) += 1;
    }

    // Get top 5 special characters
    let mut special_chars_vec: Vec<_> = special_chars.into_iter().collect();
    special_chars_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let top_special_chars = special_chars_vec.into_iter().take(5);

    // Count sentences
    let sentence_count = contents
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .count();

    // Count word frequencies and unique words
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    let mut unique_words = HashSet::new();

    for word in contents.split_whitespace() {
        let word = word
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();
        if !word.is_empty() {
            unique_words.insert(word.clone());
            *word_freq.entry(word).or_insert(0) += 1;
        }
    }

    // Get top 5 words
    let mut word_pairs: Vec<_> = word_freq.into_iter().collect();
    word_pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let top_words = word_pairs.into_iter().take(5);

    // Format output
    let mut output = String::new();
    output.push_str("=== Text Analysis ===\n\n");

    output.push_str("File Info:\n");
    output.push_str(&format!("- Size: {:.2} MB\n\n", size_mb));

    output.push_str("Pattern Counts:\n");
    output.push_str(&format!("- Emails found: {}\n", email_count));
    output.push_str(&format!("- URLs found: {}\n", url_count));
    output.push_str(&format!("- Dates found: {}\n", date_count));
    output.push_str(&format!("- Sentences: {}\n", sentence_count));
    output.push_str(&format!("- Unique words: {}\n\n", unique_words.len()));

    output.push_str("Top 5 Words:\n");
    for (word, count) in top_words {
        output.push_str(&format!("- {} ({})\n", word, count));
    }
    output.push_str("\n");

    output.push_str("Top 5 Special Characters:\n");
    for (char, count) in top_special_chars {
        output.push_str(&format!("- '{}' ({})\n", char, count));
    }
    output.push_str("\n");

    output.push_str("First 3 lines:\n");
    for line in first_three {
        output.push_str(&format!("{}\n", line));
    }
    output.push_str("\nLast 3 lines:\n");
    for line in last_three {
        output.push_str(&format!("{}\n", line));
    }

    Ok(output)
}

fn input_files(
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

    for entry in walker {
        let path = match entry {
            Ok(entry) => entry.path().to_path_buf(),
            Err(err) => {
                println!("ERROR: {}", err);
                continue;
            }
        };

        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension = extension.to_string_lossy().to_lowercase();
                if !TEXT_EXTENSIONS.contains(&extension.as_str()) {
                    continue;
                }

                let path_string = path.display().to_string();
                paths.push(path_string.clone());
                // Write contents into output
                let mut content = String::new();
                let mut file = File::open(&path)?;
                file.read_to_string(&mut content)?;
                writeln!(
                    output,
                    "<File:{}>\n{}</File:{}>\n",
                    path_string, content, path_string
                )?;
                // Store size
                let metadata = path.metadata()?;
                let size_pair = (metadata.len(), path_string);
                if sizes.len() < 6 {
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

    println!("Five biggest files: path (size)");
    for item in sizes.iter().rev() {
        println!("{} ({})", item.1, format_size(item.0));
    }

    Ok(())
}
/// Coordinates the writing of all repository content
///
/// # Arguments
/// * `repo_dir` - The repository directory path
/// * `repo` - Reference to the Git repository
/// * `exclude_patterns` - Patterns of files/directories to exclude
/// * `output` - The writer for the output
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error during writing
fn write_repo_content(
    repo_dir: &str,
    repo: &Repository,
    ignore_patterns: Vec<&str>,
    summarize_glob_patterns: Vec<String>,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    input_context(repo_dir, output)?;
    input_repo_stats(repo, output)?;
    input_files(repo_dir, output, ignore_patterns, summarize_glob_patterns)?;
    Ok(())
}
/// Main entry point for file concatenation functionality
///
/// # Arguments
/// * `repo_dir` - The repository directory path
/// * `ignore` - Comma-separated string of patterns to ignore
/// * `target` - Output file path
/// * `use_clipboard` - Whether to copy output to clipboard instead of file
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error during execution
///
/// # Details
/// Either writes the repository summary to a file or copies it to the system clipboard,
/// depending on the use_clipboard parameter
pub fn concatenate_files(
    repo_dir: &str,
    ignore: &str,
    target: &String,
    use_clipboard: bool,
    summarize: &str,
) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(repo_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let ignore_v: Vec<&str> = if ignore.is_empty() {
        Vec::new()
    } else {
        ignore.split(",").collect::<Vec<&str>>()
    };

    let mut summarize_glob_patterns: Vec<String> = Vec::new();
    if !summarize.is_empty() {
        let summarize_split = summarize.split(",");
        for pattern in summarize_split {
            let res = gitignore_to_glob(pattern);
            match res {
                Some(glob_pattern) => summarize_glob_patterns.push(glob_pattern),
                None => panic!("Invalid gitignore pattern: '{}'", pattern),
            }
        }
    }

    if use_clipboard {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        write_repo_content(
            repo_dir,
            &repo,
            ignore_v,
            summarize_glob_patterns,
            &mut cursor,
        )?;

        let content = String::from_utf8(buffer)?;
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(content.to_owned()).unwrap();
        println!("Repo contents copied to clipboard!");
    } else {
        let mut file = File::create(target)?;
        write_repo_content(
            repo_dir,
            &repo,
            ignore_v,
            summarize_glob_patterns,
            &mut file,
        )?;
        println!("Repo contents written to file: {}", target);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, Repository), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;

        {
            // Create a test file
            let test_file_path = temp_dir.path().join("test.txt");
            let mut file = File::create(&test_file_path)?;
            writeln!(file, "Test content")?;

            let mut config = repo.config()?;
            config.set_str("user.name", "Test User")?;
            config.set_str("user.email", "test@example.com")?;

            let mut index = repo.index()?;
            index.add_path(Path::new("test.txt"))?;
            index.write()?;

            {
                let tree_id = index.write_tree()?;
                let tree = repo.find_tree(tree_id)?;
                let signature = git2::Signature::now("Test User", "test@example.com")?;

                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    "Initial commit",
                    &tree,
                    &[],
                )?;
            }
        }

        Ok((temp_dir, repo)) // repo is no longer borrowed when we return it
    }

    #[test]
    fn test_input_context() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        input_context("./test-repo", &mut output)?;

        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("summary of repo ./test-repo"));
        assert!(output_str.contains("<context>"));
        assert!(output_str.contains("</context>"));
        Ok(())
    }

    #[test]
    fn test_input_repo_stats() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, repo) = setup_test_repo()?;
        let mut output = Vec::new();

        input_repo_stats(&repo, &mut output)?;

        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("<Repo statistics>"));
        assert!(output_str.contains("<Total commits>1</Total commits>"));
        assert!(output_str.contains("<First commit>"));
        assert!(output_str.contains("<Latest commits>"));
        assert!(output_str.contains("Initial commit"));

        Ok(())
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_input_files() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;

        // Create test files
        let test_file_path = temp_dir.path().join("test.txt");
        fs::write(&test_file_path, "Test content")?;

        let test_md_path = temp_dir.path().join("readme.md");
        fs::write(&test_md_path, "# Test Markdown")?;

        // Create ignored file
        let ignored_file_path = temp_dir.path().join("ignored.bin");
        fs::write(&ignored_file_path, b"binary content")?;

        let mut output = Vec::new();
        let ignore_patterns = vec!["*.bin"];

        input_files(
            temp_dir.path().to_str().unwrap(),
            &mut output,
            ignore_patterns,
            Vec::new(),
        )?;

        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("<File:"));
        assert!(output_str.contains("Test content"));
        assert!(output_str.contains("# Test Markdown"));
        assert!(!output_str.contains("binary content"));

        Ok(())
    }

    #[test]
    fn test_write_repo_content() -> Result<(), Box<dyn Error>> {
        let (temp_dir, repo) = setup_test_repo()?;
        let mut output = Vec::new();
        let ignore_patterns = vec![];

        write_repo_content(
            temp_dir.path().to_str().unwrap(),
            &repo,
            ignore_patterns,
            Vec::new(),
            &mut output,
        )?;

        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("<context>"));
        assert!(output_str.contains("<Repo statistics>"));
        assert!(output_str.contains("<File:"));

        Ok(())
    }

    #[test]
    fn test_concatenate_files() -> Result<(), Box<dyn Error>> {
        let (temp_dir, _) = setup_test_repo()?;
        let target_file = temp_dir.path().join("output.txt");

        concatenate_files(
            temp_dir.path().to_str().unwrap(),
            &String::from("*.bin"),
            &target_file.to_str().unwrap().to_string(),
            false,
            "",
        )?;

        assert!(target_file.exists());
        let content = fs::read_to_string(&target_file)?;
        assert!(content.contains("<context>"));
        assert!(content.contains("<Repo statistics>"));

        Ok(())
    }
}
