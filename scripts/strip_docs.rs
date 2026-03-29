use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

fn main() {
    let mut args = std::env::args().skip(1);
    for arg in args {
        process_dir(Path::new(&arg));
    }
}

fn process_dir(dir: &Path) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            process_dir(&path);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            process_file(&path);
        }
    }
}

fn process_file(path: &Path) {
    let content = fs::read_to_string(path).unwrap();
    let mut new_lines = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Skip doc comments
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            // Wait, does it start with /// WHY: ?
            let body = trimmed.trim_start_matches(|c| c == '/' || c == '!');
            if !body.trim().starts_with("WHY:") && !body.trim().starts_with("SAFETY:") && !is_allowed(body.trim()) {
                i += 1;
                continue;
            }
        }

        // Merge consecutive // WHY: lines
        if trimmed.starts_with("// WHY:") {
            let indent = &line[..line.len() - trimmed.len()];
            let mut why_bodies = Vec::new();
            let mut j = i;
            while j < lines.len() && lines[j].trim_start().starts_with("// WHY:") {
                let body = lines[j].trim_start().strip_prefix("// WHY:").unwrap();
                why_bodies.push(body.trim().to_string());
                j += 1;
            }
            if why_bodies.len() > 1 {
                new_lines.push(format!("{}/* WHY: {}", indent, why_bodies[0]));
                for k in 1..why_bodies.len() {
                    new_lines.push(format!("{}        {}", indent, why_bodies[k]));
                }
                new_lines.push(format!("{}   */", indent));
                i = j;
                continue;
            } else {
                new_lines.push(line.to_string());
                i += 1;
                continue;
            }
        }

        // normal line
        new_lines.push(line.to_string());
        i += 1;
    }

    // Add trailing newline if original had one or just standard
    let new_content = new_lines.join("\n") + "\n";
    if new_content != content {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(new_content.as_bytes()).unwrap();
        println!("Fixed {}", path.display());
    }
}

fn is_allowed(body: &str) -> bool {
    let clean = body.trim_end_matches("*/").trim();
    clean.is_empty()
        || clean.chars().all(|c| matches!(c, '-' | '─' | '═' | '=' | ' ' | '/' | '━' | '*'))
}
