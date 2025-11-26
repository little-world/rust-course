use std::fs;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let entries = fs::read_dir("workbook")?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "md")
        });

    for entry in entries {
        let path = entry.path();
        println!("Processing file: {}", path.display());
        if let Err(e) = process_file(&path) {
            eprintln!("Error processing file {}: {}", path.display(), e);
        }
    }

    Ok(())
}

fn process_file(path: &Path) -> std::io::Result<()> {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    fs::create_dir_all(file_stem)?;

    let content = fs::read_to_string(path)?;
    let parts: Vec<&str> = content.split("\n## Project").collect();

    if let Some(intro) = parts.get(0) {
        if !intro.trim().is_empty() {
            let intro_path = Path::new(file_stem).join("intro.md");
            fs::write(intro_path, *intro)?;
        }
    }

    for project_part in parts.iter().skip(1) {
        let full_project_content = format!("## Project{}", project_part);
        let first_line = full_project_content.lines().next().unwrap_or("").trim();

        if first_line.is_empty() {
            continue;
        }

        let filename = sanitize_filename(first_line);
        let project_path = Path::new(file_stem).join(&filename);

        fs::write(project_path, full_project_content)?;
        println!("  - Created project: {}", filename);
    }

    Ok(())
}

fn sanitize_filename(title: &str) -> String {
    let title_no_header = title.trim_start_matches("##").trim();
    let sanitized: String = title_no_header
        .to_lowercase()
        .replace(": ", "-")
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();
    format!("{}.md", sanitized)
}
