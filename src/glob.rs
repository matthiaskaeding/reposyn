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
    let pattern = if pattern.starts_with('\\') {
        &pattern[1..]
    } else {
        pattern
    };

    // Handle negation patterns - we don't convert these as standard glob
    // doesn't support negation
    if pattern.starts_with('!') {
        return None;
    }

    let mut glob = String::new();

    // Handle patterns starting with **
    if pattern.starts_with("**/") {
        glob.push_str("**/**/"); // Match in all directories
        let rest = &pattern[3..];
        glob.push_str(rest);
    }
    // Handle patterns ending with /**
    else if pattern.ends_with("/**") {
        glob.push_str(&pattern[..pattern.len() - 3]);
        glob.push_str("/**/*"); // Match everything inside
    }
    // Handle patterns with /** / in the middle
    else if pattern.contains("/**/") {
        // Split by /**/ and join with **/ to match zero or more directories
        for (i, part) in pattern.split("/**/").enumerate() {
            if i > 0 {
                glob.push_str("/**/");
            }
            glob.push_str(part);
        }
    }
    // Handle directory-only patterns (ending with /)
    else if pattern.ends_with('/') {
        glob.push_str(pattern);
        glob.push('*'); // Match directory contents
    }
    // Handle basic patterns
    else {
        // If pattern starts with /, it's relative to .gitignore location
        if pattern.starts_with('/') {
            glob.push_str("./"); // Make it relative to current directory
            glob.push_str(&pattern[1..]);
        } else {
            // Pattern can match at any level
            glob.push_str("**/");
            glob.push_str(pattern);
        }
    }

    Some(glob)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
