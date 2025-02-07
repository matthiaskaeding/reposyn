/// Converts a .gitignore pattern to a standard glob pattern.
/// Returns None for comments, empty lines, and negated patterns.
///
/// # Examples
/// ```
/// use reposyn::glob::gitignore_to_glob;
///
/// assert_eq!(gitignore_to_glob("*.txt"), Some("**/*.txt".to_string()));
/// assert_eq!(gitignore_to_glob("/file.txt"), Some("./file.txt".to_string()));
/// assert_eq!(gitignore_to_glob("dir/"), Some("dir/*".to_string()));
/// ```
pub fn gitignore_to_glob(pattern: &str) -> Option<String> {
    // Trim whitespace and handle empty lines
    let pattern = pattern.trim();
    if pattern.is_empty() || pattern.starts_with('#') {
        return None;
    }

    // Handle escaped patterns (starting with \)
    let pattern = pattern.strip_prefix('\\').unwrap_or(pattern);

    // Handle negation patterns - we don't convert these
    if pattern.starts_with('!') {
        return None;
    }

    let mut glob = String::new();

    // First handle patterns starting with /
    if let Some(rest) = pattern.strip_prefix('/') {
        glob.push_str("./");
        glob.push_str(rest);
        if glob.ends_with('/') {
            glob.push('*');
        }
        return Some(glob);
    }

    // Handle patterns with **
    if let Some(rest) = pattern.strip_prefix("**/") {
        glob.push_str("**/**/");
        glob.push_str(rest);
    } else if let Some(rest) = pattern.strip_suffix("/**") {
        glob.push_str(rest);
        glob.push_str("/**/*");
    } else if pattern.contains("/**/") {
        // Keep the original pattern for /**/ cases
        glob.push_str(pattern);
    } else if pattern.ends_with('/') {
        glob.push_str(pattern);
        glob.push('*');
    } else {
        // Pattern can match at any level
        glob.push_str("**/");
        glob.push_str(pattern);
    }

    Some(glob)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glob_match::glob_match;

    #[test]
    fn test_basic_patterns() {
        assert_eq!(gitignore_to_glob("*.txt"), Some("**/*.txt".to_string()));
        assert_eq!(
            gitignore_to_glob("/file.txt"),
            Some("./file.txt".to_string())
        );
        assert_eq!(gitignore_to_glob("dir/"), Some("dir/*".to_string()));
    }

    #[test]
    fn test_double_asterisk_patterns() {
        assert_eq!(gitignore_to_glob("**/foo"), Some("**/**/foo".to_string()));
        assert_eq!(gitignore_to_glob("abc/**"), Some("abc/**/*".to_string()));
        assert_eq!(gitignore_to_glob("a/**/b"), Some("a/**/b".to_string()));
    }

    #[test]
    fn test_special_cases() {
        assert_eq!(gitignore_to_glob(""), None);
        assert_eq!(gitignore_to_glob("#comment"), None);
        assert_eq!(gitignore_to_glob("!excluded"), None);
        assert_eq!(
            gitignore_to_glob("\\#notacomment"),
            Some("**/#notacomment".to_string())
        );
    }

    #[test]
    fn test_with_glob_match() {
        let pattern = gitignore_to_glob("*.toml");
        assert_eq!(pattern, Some("**/*.toml".to_string()));
        match pattern {
            Some(pattern) => {
                let is_match = glob_match(pattern.as_str(), "Cargo.toml");
                assert_eq!(is_match, true);
            }
            None => (),
        }
    }

    #[test]
    fn test_invalid_patterns() {
        assert_eq!(gitignore_to_glob(""), None);
        assert_eq!(gitignore_to_glob("  "), None);
        assert_eq!(gitignore_to_glob("\t"), None);
        assert_eq!(gitignore_to_glob("#"), None);
        assert_eq!(gitignore_to_glob("# comment"), None);
        assert_eq!(gitignore_to_glob("!*.toml"), None);
    }

    #[test]
    fn test_extended_patterns() {
        // Basic patterns with different extensions
        assert_eq!(gitignore_to_glob("*.toml"), Some("**/*.toml".to_string()));
        assert_eq!(gitignore_to_glob("*.rs"), Some("**/*.rs".to_string()));
        assert_eq!(
            gitignore_to_glob("Cargo.toml"),
            Some("**/Cargo.toml".to_string())
        );

        // Paths with multiple segments
        assert_eq!(
            gitignore_to_glob("src/*.rs"),
            Some("**/src/*.rs".to_string())
        );
        assert_eq!(
            gitignore_to_glob("/src/*.rs"),
            Some("./src/*.rs".to_string())
        );

        // Directory patterns
        assert_eq!(
            gitignore_to_glob("node_modules/"),
            Some("node_modules/*".to_string())
        );
        assert_eq!(
            gitignore_to_glob("/node_modules/"),
            Some("./node_modules/*".to_string())
        );

        // Complex patterns
        assert_eq!(
            gitignore_to_glob("src/**/test/*.rs"),
            Some("src/**/test/*.rs".to_string())
        );
        assert_eq!(
            gitignore_to_glob("**/src/test.rs"),
            Some("**/**/src/test.rs".to_string())
        );
        // Changed this line to match the more correct pattern
        assert_eq!(
            gitignore_to_glob("build/**/*.js"),
            Some("build/**/*.js".to_string())
        );

        // Special characters
        assert_eq!(
            gitignore_to_glob("*.{js,ts}"),
            Some("**/*.{js,ts}".to_string())
        );
        assert_eq!(
            gitignore_to_glob("[abc]*.rs"),
            Some("**/[abc]*.rs".to_string())
        );

        // Whitespace handling
        assert_eq!(gitignore_to_glob(" *.toml "), Some("**/*.toml".to_string()));
        assert_eq!(gitignore_to_glob("\t*.rs\n"), Some("**/*.rs".to_string()));
    }
    #[test]
    fn test_path_separators() {
        // Test with different path separators
        assert_eq!(
            gitignore_to_glob("src/test"),
            Some("**/src/test".to_string())
        );
        assert_eq!(
            gitignore_to_glob("src\\test"),
            Some("**/src\\test".to_string())
        );
        assert_eq!(
            gitignore_to_glob("/src/test"),
            Some("./src/test".to_string())
        );
    }
}
