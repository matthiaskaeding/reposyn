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
    if config.use_clipboard {
        let mut buffer = Vec::new();
        let mut output = Cursor::new(&mut buffer);
        write_repo_content(config, &mut output)?;

        let content = String::from_utf8(buffer)?;
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(content.to_owned()).unwrap();

        let duration = config.created_at.elapsed();
        println!(
            "Repo contents copied to clipboard. Took {:.2}s",
            duration.as_secs_f64()
        );
    } else {
        let mut file = File::create(&config.output_file)?;
        write_repo_content(config, &mut file)?;
        let duration = config.created_at.elapsed();
        println!(
            "Repo contents written to file: {}. Took {:.2}s",
            &config.output_file.display(),
            duration.as_secs_f64()
        );
    }

    Ok(())
}

/// Coordinates the writing of all repository content
///
/// # Arguments
/// * `config` - Configuration
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
    use crate::test_utils::setup_test_repo;
    use std::error::Error;
    use std::fs;

    #[test]
    fn test_merge_files() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, config) = setup_test_repo()?;

        merge_files(&config)?;

        assert!(config.output_file.exists());
        let content = fs::read_to_string(config.output_file)?;
        assert!(content.contains("<context>"));
        assert!(content.contains("<Repo statistics>"));
        assert!(content.contains("Test content")); // Content from our test file

        Ok(())
    }

    #[test]
    fn test_merge_files_with_summarize() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, mut config) = setup_test_repo()?;

        // Add some additional test files
        fs::write(config.repo_path.join("test1.json"), r#"{"key": "value"}"#)?;
        fs::write(
            config.repo_path.join("test2.json"),
            r#"{"another": "value"}"#,
        )?;

        // Update config to summarize JSON files
        config.summarize_patterns = vec!["*.json".to_string()];
        config.text_extensions.insert("json".to_string());

        merge_files(&config)?;

        let content = fs::read_to_string(&config.output_file)?;

        // Check that JSON files were summarized
        assert!(content.contains("Summary of file:"));
        assert!(content.contains("test1.json"));
        assert!(content.contains("test2.json"));

        // Original text file should be included normally
        assert!(content.contains("Test content"));

        Ok(())
    }

    #[test]
    fn test_merge_files_clipboard() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, mut config) = setup_test_repo()?;
        config.use_clipboard = true;

        merge_files(&config)?;
        // Note: We can't easily test clipboard content in a unit test
        // This test mainly ensures the function runs without errors

        Ok(())
    }
}
