use std::env;
use std::fs;
use std::io;
use std::path::Path;
use regex::Regex;

/// Cleans a single line if it’s a Markdown header.
fn clean_header_line(line: &str,
                     non_ascii: &Regex,
                     trash_markers: &Regex,
                     number_pattern: &Regex)
                     -> String {
    // Matches e.g. "## 🧑‍💻 3. **Write Your First Program**"
    let header_re = Regex::new(r"^(?P<prefix>#+\s*)(?P<title>.*)$").unwrap();
    if let Some(caps) = header_re.captures(line) {
        let prefix = caps.name("prefix").unwrap().as_str();
        let mut title = caps.name("title").unwrap().as_str().to_string();

        // 1. remove non-ASCII (emojis etc.)
        title = non_ascii.replace_all(&title, "").to_string();
        // 2. remove *, _, and backticks
        title = trash_markers.replace_all(&title, "").to_string();
        // 3. remove leading numbers + optional dot (e.g. "3." or "12.")
        title = number_pattern.replace_all(&title, "").to_string();
        // 4. collapse any repeated whitespace into single spaces, then trim
        let title = title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        format!("{}{}", prefix, title)
    } else {
        // non-header lines are left intact
        line.to_string()
    }
}

/// Reads a file, processes it line-by-line, and overwrites it with the cleaned version.
fn process_file(path: &Path,
                non_ascii: &Regex,
                trash_markers: &Regex,
                number_pattern: &Regex)
                -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    let cleaned = content
        .lines()
        .map(|line| clean_header_line(line, non_ascii, trash_markers, number_pattern))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, cleaned)?;
    Ok(())
}

fn main() -> io::Result<()> {
    // expect exactly one argument: the directory to scan
  
    let dir = "./examples/docs/std-util";

    let entries = fs::read_dir(dir)?;
    println!("{:#?}", entries.collect::<Vec<_>>());
    // compile the regexes once
    let non_ascii      = Regex::new(r"[^\x00-\x7F]+").unwrap();
    let trash_markers  = Regex::new(r#"[*_`]"#).unwrap();
    let number_pattern = Regex::new(r"\d+\.?").unwrap();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            process_file(&path, &non_ascii, &trash_markers, &number_pattern)?;
            println!("Processed {}", path.display());
        }
    }

    Ok(())
}