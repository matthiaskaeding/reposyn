# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "textstat",
# ]
# ///

# %%
import textstat
from pathlib import Path
from collections import Counter
import re
from datetime import datetime
import mimetypes
import os


file = Path("requests/HISTORY.md")
assert file.exists()
text = file.read_text()




# %%
def summarize_file(filepath):
    file_stats = os.stat(filepath)
    file_size = file_stats.st_size
    mime_type = mimetypes.guess_type(filepath)[0]

    # Calculate file hash (first 1MB for large files)
    with open(filepath, "rb") as f:

        # Process as text
        f.seek(0)
        text = f.read()
        if isinstance(text, bytes):
            text = text.decode("utf-8", errors="ignore")

        # Get language features
        sentences = textstat.sentence_count(text)
        words = text.split()

        # Word frequency analysis (top 10)
        word_freq = Counter(re.findall(r"\b\w+\b", text.lower()))

        # Detect potential patterns
        emails = len(re.findall(r"[\w\.-]+@[\w\.-]+\.\w+", text))
        urls = len(
            re.findall(
                r"http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+",
                text,
            )
        )
        dates = len(re.findall(r"\d{1,2}[-/]\d{1,2}[-/]\d{2,4}", text))

        return {
            "file_info": {
                "type": "text",
                "size_bytes": file_size,
                "size_mb": round(file_size / (1024 * 1024), 2),
                "mime_type": mime_type,
                "last_modified": datetime.fromtimestamp(
                    file_stats.st_mtime
                ).isoformat(),
                "encoding": "utf-8",
            },
            "content_summary": {
                "length": len(text),
                "lines": text.count("\n") + 1,
                "sentences": sentences,
                "words": len(words),
                "unique_words": len(set(words)),
                "chars_per_word": round(len(text) / len(words), 2) if words else 0,
                "start": text[:500] + "..." if len(text) > 500 else text,
                "end": text[-500:] + "..." if len(text) > 500 else "",
            },
            "readability": {
                "reading_time_seconds": textstat.reading_time(text),
                "reading_ease": textstat.flesch_reading_ease(text),
                "grade_level": textstat.coleman_liau_index(text),
                "complex_word_ratio": textstat.difficult_words(text) / len(words)
                if words
                else 0,
            },
            "patterns": {
                "emails_found": emails,
                "urls_found": urls,
                "dates_found": dates,
                "numbers_found": len(re.findall(r"\b\d+\b", text)),
            },
            "frequency_analysis": {
                "top_words": dict(word_freq.most_common(10)),
                "special_chars": dict(
                    Counter(re.findall(r"[^a-zA-Z0-9\s]", text)).most_common(5)
                ),
            },
        }


summarize_file(file)
