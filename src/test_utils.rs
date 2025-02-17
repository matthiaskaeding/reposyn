use crate::config::RepoConfig;
use git2::Repository;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

pub(crate) fn setup_test_repo() -> Result<(TempDir, RepoConfig), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    let repo = Repository::init(&repo_path)?;

    // Create a test file
    let test_file_path = repo_path.join("test.txt");
    fs::write(&test_file_path, "Test content")?;

    // Set up git config
    let mut config = repo.config()?;
    config.set_str("user.name", "Test User")?;
    config.set_str("user.email", "test@example.com")?;

    // Add and commit the file
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("test.txt"))?;
    index.write()?;

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
    let output_file = temp_dir
        .path()
        .join("test_output.txt")
        .to_string_lossy()
        .into_owned();
    let config = RepoConfig::new(
        repo_path,
        false,
        output_file,
        Vec::new(),
        Vec::new(),
        HashSet::from(["txt".to_string()]),
        Instant::now(),
        500.0,
    )?;

    Ok((temp_dir, config))
}
