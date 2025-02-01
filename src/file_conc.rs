use git2::Repository;
use ignore::Walk;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use time::OffsetDateTime;

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "json", "yaml", "yml", "toml", "css", "html", "htm", "xml",
    "csv", "sh", "bash", "conf",
];

fn input_context(input_dir: &str, output: &mut File) -> Result<(), Box<dyn Error>> {
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

    writeln!(output, "<Repo statistics>")?;
    if let (Some(first), Some(latest)) = (first_time, latest_time) {
        let first_date = OffsetDateTime::from_unix_timestamp(first)?;
        let latest_date = OffsetDateTime::from_unix_timestamp(latest)?;
        writeln!(
            output,
            "<Total commits>{}</Total commits>",
            count.to_string()
        )?;
        writeln!(
            output,
            "<First commit>{}</First commit>",
            first_date.to_string()
        )?;
        writeln!(
            output,
            "<Latest commits>{}</Latest commits>",
            latest_date.to_string()
        )?;
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
            i.to_string(),
            msg_clean,
            i.to_string(),
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
    output: &mut File,
    ignore_patterns: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut sizes = BTreeSet::new();

    for entry in Walk::new(repo_dir) {
        let path = match entry {
            Ok(entry) => entry.path().to_path_buf(),
            Err(err) => {
                println!("ERROR: {}", err);
                continue;
            }
        };

        if ignore_patterns
            .iter()
            .any(|pattern| path.to_string_lossy().contains(pattern))
        {
            continue;
        }

        if path.is_file() {
            let metadata = path.metadata()?;

            let size_pair = (metadata.len(), path.display().to_string());
            if sizes.len() < 6 {
                sizes.insert(size_pair);
            } else if let Some(smallest) = sizes.first().cloned() {
                if size_pair > smallest {
                    sizes.remove(&smallest); // Remove the smallest element
                    sizes.insert(size_pair);
                }
            }

            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if TEXT_EXTENSIONS.contains(&ext.as_str()) {
                    let mut content = String::new();
                    let mut file = File::open(&path)?;
                    file.read_to_string(&mut content)?;
                    writeln!(
                        output,
                        "<File:{}>\n{}</File:{}>\n",
                        path.display().to_string(),
                        content,
                        path.display().to_string()
                    )?;
                }
            }
        }
    }

    println!("Five biggest files: path (size)");
    for item in sizes.iter().rev() {
        println!("{} ({})", item.1, format_size(item.0));
    }
    Ok(())
}

pub fn concatenate_files(
    repo_dir: &str,
    exclude_patterns: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(repo_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let mut output = File::create("repo-synopsis.txt")?;
    input_context(&repo_dir, &mut output)?;
    input_repo_stats(&repo, &mut output)?;
    input_files(repo_dir, &mut output, exclude_patterns)?;
    Ok(())
}
