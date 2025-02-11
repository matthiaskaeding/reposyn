use git2::Repository;
use std::collections::HashSet;

#[non_exhaustive]
pub struct RepoConfig {
    pub repo_name: String,
    pub repo_path: std::path::PathBuf,
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
        repo_path: std::path::PathBuf,
        use_clipboard: bool,
        output_file: String,
        ignore_patterns: Vec<String>,
        summarize_patterns: Vec<String>,
        text_extensions: HashSet<String>,
        created_at: std::time::Instant,
        thr: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Check if path exists
        if !repo_path.exists() {
            return Err(format!("Path does not exist: {}", repo_path.display()).into());
        }

        // Get repo name from path
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
