use git2::Repository;
use std::collections::HashSet;
use std::path::PathBuf;

fn find_git_repository(start_dir: &PathBuf) -> Option<std::path::PathBuf> {
    let mut current_dir = start_dir.clone();

    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.is_dir() {
            return Some(current_dir);
        }

        if !current_dir.pop() {
            return None;
        }
    }
}

#[non_exhaustive]
pub struct RepoConfig {
    pub repo_name: String,
    pub repo_path: std::path::PathBuf,
    pub input_folder: std::path::PathBuf,
    pub use_clipboard: bool,
    pub output_file: std::path::PathBuf,
    pub ignore_patterns: Vec<String>,
    pub summarize_patterns: Vec<String>,
    pub created_at: std::time::Instant,
    pub thr: f64,
    pub text_extensions: HashSet<String>,
}
#[allow(clippy::too_many_arguments)]
impl RepoConfig {
    pub fn new(
        use_clipboard: bool,
        input_folder: String,
        output_file: String,
        ignore_patterns: Vec<String>,
        summarize_patterns: Vec<String>,
        text_extensions: HashSet<String>,
        created_at: std::time::Instant,
        thr: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let input_folder = std::path::PathBuf::from(input_folder);
        if !input_folder.exists() {
            panic!("Path does not exist: {}", input_folder.display());
        }
        let repo_path = find_git_repository(&input_folder)
            .expect("No git repo found in this current directory or above it");

        println!("Found repo_path at {}", repo_path.display());

        let repo_name = if let Some(name) = repo_path.file_name() {
            if let Some(name_str) = name.to_str() {
                name_str.to_string()
            } else {
                repo_path.to_string_lossy().into_owned()
            }
        } else {
            repo_path.to_string_lossy().into_owned()
        };
        let output_file = std::path::PathBuf::from(output_file);
        Ok(Self {
            input_folder,
            repo_name,
            repo_path,
            use_clipboard,
            output_file,
            ignore_patterns,
            summarize_patterns,
            created_at,
            thr,
            text_extensions,
        })
    }

    pub fn open_repo(&self) -> Result<Repository, Box<dyn std::error::Error>> {
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open git repository: {}", e))?;
        Ok(repo)
    }
}
