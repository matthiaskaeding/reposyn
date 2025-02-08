use crate::config::RepoConfig;
use crate::input::{input_context, input_files, input_repo_stats};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

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
pub fn merge_files(config: &RepoConfig) -> Result<(), Box<dyn Error>> {
    let start_time = std::time::Instant::now();

    if config.use_clipboard {
        let mut buffer = Vec::new();
        let mut output = Cursor::new(&mut buffer);
        write_repo_content(config, &mut output)?;

        let content = String::from_utf8(buffer)?;
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(content.to_owned()).unwrap();

        let duration = start_time.elapsed();
        println!(
            "Repo contents copied to clipboard. Took {:.2}s",
            duration.as_secs_f64()
        );
    } else {
        let mut file = File::create(&config.output_file)?;
        write_repo_content(config, &mut file)?;
        let duration = start_time.elapsed();
        println!(
            "Repo contents written to file: {}. Took {:.2}s",
            &config.output_file,
            duration.as_secs_f64()
        );
    }

    Ok(())
}

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
fn write_repo_content(config: &RepoConfig, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    input_context(&config.repo_name, output)?;
    input_repo_stats(&config.repo, output)?;
    input_files(config, output)?;

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
        assert!(output_str.contains("./test-repo"));
        assert!(output_str.contains("engineer"));
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

    #[test]
    fn test_merge_files_with_summarize() -> Result<(), Box<dyn Error>> {
        let (temp_dir, _) = setup_test_repo()?;

        // Create a few different file types
        fs::write(temp_dir.path().join("test1.json"), r#"{"key": "value"}"#)?;
        fs::write(
            temp_dir.path().join("test2.json"),
            r#"{"another": "value"}"#,
        )?;
        fs::write(temp_dir.path().join("normal.txt"), "Regular text file")?;

        let target_file = temp_dir.path().join("output.txt");

        // Test summarizing all JSON files
        merge_files(
            temp_dir.path().to_str().unwrap(),
            "", // no ignore patterns
            &target_file.to_str().unwrap().to_string(),
            false,
            "*.json", // summarize all JSON files
        )?;

        let content = fs::read_to_string(&target_file)?;

        // Check that JSON files were summarized
        assert!(content.contains("Summary of file:"));
        assert!(content.contains("test1.json"));
        assert!(content.contains("test2.json"));

        // The raw JSON content should not be present
        assert!(!content.contains("<File:test1.json>"));

        // Regular text file should be included normally
        assert!(content.contains("Regular text file"));

        Ok(())
    }

    #[test]
    fn test_merge_files_multiple_summarize_patterns() -> Result<(), Box<dyn Error>> {
        println!("Starting multiple summarize patterns test");
        let (temp_dir, _) = setup_test_repo()?;
        println!("Test repo set up at: {:?}", temp_dir.path());

        // Create different types of files
        fs::write(temp_dir.path().join("data.json"), r#"{"data": "test"}"#)?;
        fs::write(temp_dir.path().join("config.yaml"), "key: value")?;
        fs::write(temp_dir.path().join("readme.md"), "# Title")?;
        println!("Created test files");

        let target_file = temp_dir.path().join("output.txt");
        println!("Target file will be: {:?}", target_file);

        // Test summarizing multiple file types
        println!("Calling merge_files with patterns: *.json,*.yaml");
        merge_files(
            temp_dir.path().to_str().unwrap(),
            "",
            &target_file.to_str().unwrap().to_string(),
            false,
            "*.json,*.yaml", // summarize both JSON and YAML files
        )?;

        println!("Reading content from target file");
        let content = fs::read_to_string(&target_file)?;
        println!(
            "\n--- BEGIN CONTENT ---\n{}\n--- END CONTENT ---\n",
            content
        );

        // Check that both JSON and YAML files were summarized
        println!("Checking for summaries");
        assert!(content.contains("<Summary of file:") && content.contains("data.json"));
        assert!(content.contains("<Summary of file:") && content.contains("config.yaml"));

        // Raw content of summarized files should not be present
        println!("Checking files not present");

        assert!(!content.contains("<File:data.json>"));

        // Markdown file should be included normally
        println!("Checking markdown content");
        assert!(content.contains("# Title"));

        Ok(())
    }
}
