use git2::Repository;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use time::OffsetDateTime;
use ignore::Walk;

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "json", "yaml", "yml", "toml",
    "css", "html", "htm", "xml", "csv", "sh", "bash", "conf",
];

fn input_repo_stats(repo: &Repository, output: &mut File) -> Result<(), Box<dyn Error>> {
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
    writeln!(output, "\n# Files")?;

    for entry in Walk::new(repo_dir) {
        let path = match entry {
            Ok(entry) => entry.path().to_path_buf(),
            Err(err) => {
                println!("ERROR: {}", err);
                continue;
            }
        };

        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if TEXT_EXTENSIONS.contains(&ext.as_str()) {
                    let mut content = String::new();
                    let mut file = File::open(&path)?;
                    file.read_to_string(&mut content)?;

                    writeln!(output, "\n## File: {}", path.display())?;
                    write!(output, "{}\n", content)?;
                    writeln!(output)?;
                } 
            }
        }
    }

    Ok(())
}

pub fn concatenate_files(repo_dir: &str) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(repo_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let mut output = File::create("repo-synopsis.md")?;
    writeln!(output, "# Input dir: {}", repo_dir)?;

    let _ = input_repo_stats(&repo, &mut output);
    let _ = input_files(repo_dir, &mut output);
    Ok(())
}
