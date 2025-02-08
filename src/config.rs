use git2::Repository;
use std::collections::HashSet;

pub struct RepoConfig {
    pub repo_dir: String,
    pub repo_name: String,
    pub repo: Repository,
    pub use_clipboard: bool,
    pub output_file: String,
    pub ignore_patterns: Vec<String>,
    pub summarize_patterns: Vec<String>,

    pub text_extensions: HashSet<String>,
}
