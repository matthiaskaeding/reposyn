use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

fn summarize_text_file(filepath: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    // Read file contents and get file size
    let mut file = File::open(filepath)?;
    let metadata = std::fs::metadata(filepath)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0); // Convert bytes to MB

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Get first and last 3 lines
    let reader = BufReader::new(File::open(filepath)?);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let first_three = lines.iter().take(3).cloned().collect::<Vec<_>>();
    let last_three = lines
        .iter()
        .rev()
        .take(3)
        .rev()
        .cloned()
        .collect::<Vec<_>>();

    // Compile regex patterns
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

    // Format output
    let mut output = String::new();
    output.push_str("=== Text Analysis ===\n\n");

    output.push_str("File Info:\n");
    output.push_str(&format!("- Size: {:.2} MB\n\n", size_mb));

    output.push_str("Pattern Counts:\n");
    output.push_str(&format!("- Emails found: {}\n", email_count));
    output.push_str(&format!("- URLs found: {}\n", url_count));
    output.push_str(&format!("- Dates found: {}\n", date_count));
    output.push_str(&format!("- Sentences: {}\n", sentence_count));
    output.push_str(&format!("- Unique words: {}\n\n", unique_words.len()));

    output.push_str("Top 5 Words:\n");
    for (word, count) in top_words {
        output.push_str(&format!("- {} ({})\n", word, count));
    }
    output.push('\n');

    output.push_str("Top 5 Special Characters:\n");
    for (char, count) in top_special_chars {
        output.push_str(&format!("- '{}' ({})\n", char, count));
    }
    output.push('\n');

    output.push_str("First 3 lines:\n");
    for line in first_three {
        output.push_str(&format!("{}\n", line));
    }
    output.push_str("\nLast 3 lines:\n");
    for line in last_three {
        output.push_str(&format!("{}\n", line));
    }

    Ok(output)
}
