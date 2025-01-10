use git2::Repository;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use time::OffsetDateTime;

fn input_repo_stats(repo: &Repository, output: &mut File) -> Result<(), Box<dyn Error>> {
    // Setup revwalk to iterate through commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    // Get total commits and timestamps
    let mut count = 0;
    let mut first_time = None;
    let mut latest_time = None;
    let mut recent_messages = Vec::new();

    for commit_id in revwalk {
        let commit = repo.find_commit(commit_id?)?;
        let time = commit.time().seconds();

        // Update latest time (first commit in walk)
        if count == 0 {
            latest_time = Some(time);
        }
        if count < 3 {
            if let Some(msg) = commit.message() {
                recent_messages.push(msg.to_string());
            }
        }

        // Always update first time (will end up with last commit in walk)
        first_time = Some(time);
        count += 1;
    }

    writeln!(output, "# Repo stats")?;

    // Convert timestamps to readable dates
    if let (Some(first), Some(latest)) = (first_time, latest_time) {
        let first_date = OffsetDateTime::from_unix_timestamp(first)?;
        let latest_date = OffsetDateTime::from_unix_timestamp(latest)?;

        writeln!(output, "Total commits: {}", count.to_string())?;
        writeln!(output, "First commit: {}", first_date.to_string())?;
        writeln!(output, "Latest commits: {}\n", latest_date.to_string())?;
    }

    writeln!(output, "# Last three commit messages")?;
    for (i, msg) in recent_messages.iter().enumerate() {
        writeln!(output, "## Commit msg({})\n{}\n", i.to_string(), msg)?;
    }

    Ok(())
}

fn input_files(repo_dir: &str, output: &mut File) -> Result<(), Box<dyn Error>> {
    // Read the directory
    let entries = fs::read_dir(repo_dir)?;

    // Process each file in the directory
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a file and has the correct extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "py" || ext == "txt" {
                    // Read the file content
                    let mut content = String::new();
                    let mut file = File::open(&path)?;
                    file.read_to_string(&mut content)?;

                    // Write file name as a comment
                    writeln!(output, "\n## File: {}", path.display())?;

                    // Write the content
                    write!(output, "{}", content)?;

                    // Add a newline for separation
                    writeln!(output)?;
                }
            }
        }
    }

    Ok(())
}

pub fn concatenate_files(repo_dir: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Get repo
    let repo = match Repository::open(repo_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    // Create or truncate the output file
    let mut output = File::create(output_file)?;
    writeln!(output, "# Input dir: {}", repo_dir)?;

    // Git info
    let _ = input_repo_stats(&repo, &mut output);
    let _ = input_files(repo_dir, &mut output);
    Ok(())
}
