use copypasta::{ClipboardContext, ClipboardProvider};
use git2::Repository;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::{Read, Write};
use std::path::Path;
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

fn input_files(
    repo_dir: &str,
    output: &mut impl Write,
    ignore_patterns: Vec<&str>,
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
    exclude_patterns: Vec<&str>,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    input_context(repo_dir, output)?;
    input_repo_stats(repo, output)?;
    input_files(repo_dir, output, exclude_patterns)?;
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
    if use_clipboard {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        write_repo_content(repo_dir, &repo, ignore_v, &mut cursor)?;

        let content = String::from_utf8(buffer)?;
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(content.to_owned()).unwrap();
        println!("Repo contents copied to clipboard!");
    } else {
        let mut file = File::create(target)?;
        write_repo_content(repo_dir, &repo, ignore_v, &mut file)?;
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
        )?;

        assert!(target_file.exists());
        let content = fs::read_to_string(&target_file)?;
        assert!(content.contains("<context>"));
        assert!(content.contains("<Repo statistics>"));

        Ok(())
    }
}
