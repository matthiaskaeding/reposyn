use crate::config::RepoConfig;
use crate::input::{input_context, input_files, input_repo_stats};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::{BufWriter, Write};
use std::time::Duration;

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
    let mut target_string = "".to_string();
    let durations: WriteDurations;
    let duration_total: Duration;
    if config.use_clipboard {
        target_string.push_str("clipboard");
        let mut buffer = Vec::new();
        let mut output = Cursor::new(&mut buffer);
        durations = write_repo_content(config, &mut output)?;

        let content = String::from_utf8(buffer)?;
        let mut ctx = ClipboardContext::new().unwrap();
        ctx.set_contents(content.to_owned()).unwrap();

        duration_total = config.created_at.elapsed();
    } else {
        target_string.push_str(&format!("file {}", &config.output_file.display()));
        let file = File::create(&config.output_file)?;
        let mut output = BufWriter::new(file);
        durations = write_repo_content(config, &mut output)?;
        output.flush()?;
        duration_total = config.created_at.elapsed();
    }

    let duration_total_seconds = duration_total.as_secs_f64();

    println!(
        "Repo contents copied to {} in {:.1}s. {:.1}s for file merging, {:.1}s for repo statistics",
        target_string,
        duration_total_seconds,
        durations.files.as_secs_f64(),
        durations.stats.as_secs_f64()
    );

    Ok(())
}
pub struct WriteDurations {
    pub files: Duration,
    pub stats: Duration,
}
/// Coordinates the writing of all repository content
///
/// # Arguments
/// * `config` - Configuration
/// * `output` - The writer for the output
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error during writing
fn write_repo_content(
    config: &RepoConfig,
    output: &mut impl Write,
) -> Result<WriteDurations, Box<dyn Error>> {
    input_context(&config.repo_name, output)?;
    let mut now = std::time::Instant::now();
    input_repo_stats(config, output)?;
    let duration_stats = now.elapsed();
    now = std::time::Instant::now();
    input_files(config, output)?;
    let duration_files = now.elapsed();

    Ok(WriteDurations {
        files: duration_files,
        stats: duration_stats,
    })
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
        assert!(content.contains("Test content"));

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
    #[cfg(not(feature = "ci-tests"))] // Only run when ci-tests feature is not enabled
    fn test_merge_files_clipboard() -> Result<(), Box<dyn Error>> {
        let (_temp_dir, mut config) = setup_test_repo()?;
        config.use_clipboard = true;

        merge_files(&config)?;
        // Note: We can't easily test clipboard content in a unit test
        // This test mainly ensures the function runs without errors

        Ok(())
    }
}
