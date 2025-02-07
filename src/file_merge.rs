use crate::glob::gitignore_to_glob;
use crate::input::{input_context, input_files, input_repo_stats};
use copypasta::{ClipboardContext, ClipboardProvider};
use git2::Repository;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

/// Coordinates the writing of all repository content
///
/// # Arguments
/// * `repo_dir` - The repository directory path
/// * `repo` - Reference to the Git repository
/// * `exclude_patterns` - Patterns of files/directories to exclude
/// * `summarize_glob_patterns` - Patterns of files to summarize
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
pub fn merge_files(
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
                Some(glob_pattern) => {
                    // We'll need to ensure the glob_pattern has a lifetime that matches the Vec
                    // This might require changes to gitignore_to_glob's return type
                    // or storing the patterns differently depending on your use case
                    summarize_glob_patterns.push(glob_pattern)
                }
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
    use crate::input::format_size;
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
    fn test_merge_files() -> Result<(), Box<dyn Error>> {
        let (temp_dir, _) = setup_test_repo()?;
        let target_file = temp_dir.path().join("output.txt");

        merge_files(
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
