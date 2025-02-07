// Input summary of a text file
// Might be extended later
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn input_summary(
    filepath: &PathBuf,
    output: &mut impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read file contents and get file size
    let mut file = File::open(filepath)?;
    let metadata = std::fs::metadata(filepath)?;

    writeln!(output, "<Summary of file: {:?}>", filepath)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0); // Convert bytes to MB
    writeln!(output, "Size in mb: {}", size_mb)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Get first and last 3 lines
    let lines: Vec<&str> = contents.lines().collect();

    writeln!(output, "<First 3 lines>")?;
    for line in lines.iter().take(3) {
        writeln!(output, "{}", line)?;
    }
    writeln!(output, "</First 3 lines>")?;

    writeln!(output, "<Last 3 lines>")?;
    if lines.len() > 3 {
        let lines_to_write = lines.iter().skip(lines.len() - 3);
        for line in lines_to_write {
            writeln!(output, "{}", line)?;
        }
    }
    writeln!(output, "</Last 3 lines>")?;

    // Count occurences
    let email_regex = Regex::new(r"[\w\.-]+@[\w\.-]+\.\w+")?;
    let url_regex = Regex::new(
        r"http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+",
    )?;
    let date_regex = Regex::new(r"\d{1,2}[-/]\d{1,2}[-/]\d{2,4}")?;
    let special_char_regex = Regex::new(r"[^a-zA-Z0-9\s]")?;

    // Count patterns
    let email_count = email_regex.find_iter(&contents).count();
    let url_count = url_regex.find_iter(&contents).count();
    let date_count = date_regex.find_iter(&contents).count();

    // Count special characters
    let mut special_chars: HashMap<String, usize> = HashMap::new();
    for special_char in special_char_regex.find_iter(&contents) {
        let char_str = special_char.as_str().to_string();
        *special_chars.entry(char_str).or_insert(0) += 1;
    }

    // Get top 5 special characters
    let mut special_chars_vec: Vec<_> = special_chars.into_iter().collect();
    special_chars_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let top_special_chars = special_chars_vec.into_iter().take(5);

    // Count sentences
    let sentence_count = contents
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .count();

    // Count word frequencies and unique words
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    let mut unique_words = HashSet::new();

    for word in contents.split_whitespace() {
        let word = word
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();
        if !word.is_empty() {
            unique_words.insert(word.clone());
            *word_freq.entry(word).or_insert(0) += 1;
        }
    }

    // Get top 5 words
    let mut word_pairs: Vec<_> = word_freq.into_iter().collect();
    word_pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let top_words = word_pairs.into_iter().take(5);

    writeln!(output, "File Info:")?;
    writeln!(output, "- Size: {:.2} MB\n", size_mb)?;

    writeln!(output, "Pattern Counts:")?;
    writeln!(output, "- Emails found: {}", email_count)?;
    writeln!(output, "- URLs found: {}", url_count)?;
    writeln!(output, "- Dates found: {}", date_count)?;
    writeln!(output, "- Sentences: {}", sentence_count)?;
    writeln!(output, "- Unique words: {}\n", unique_words.len())?;

    writeln!(output, "Top 5 Words:")?;
    for (word, count) in top_words {
        writeln!(output, "- {} ({})", word, count)?;
    }
    writeln!(output)?;

    writeln!(output, "Top 5 Special Characters:")?;
    for (char, count) in top_special_chars {
        writeln!(output, "- '{}' ({})", char, count)?;
    }
    writeln!(output)?;

    // Done
    writeln!(output, "</Summary of file: {:?}>", filepath)?;
    Ok(())
}
