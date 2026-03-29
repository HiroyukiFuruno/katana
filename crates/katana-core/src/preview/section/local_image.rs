use super::PreviewSection;
use regex::Regex;

pub fn extract_standalone_images(initial_sections: Vec<PreviewSection>) -> Vec<PreviewSection> {
    let img_re = Regex::new(r"(?m)^[ \t]*!\[([^\]]*)\]\(([^\)]+)\)[ \t]*$").unwrap();
    let mut temp = Vec::new();

    for sec in initial_sections {
        match sec {
            PreviewSection::Markdown(text) => process_markdown_section(&text, &img_re, &mut temp),
            other => temp.push(other),
        }
    }
    temp
}

fn process_markdown_section(text: &str, img_re: &Regex, temp: &mut Vec<PreviewSection>) {
    let mut last_end = 0;
    for cap in img_re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let before = &text[last_end..m.start()];
        if !before.trim().is_empty() {
            temp.push(PreviewSection::Markdown(before.to_string()));
        }
        temp.push(PreviewSection::LocalImage {
            path: cap.get(2).unwrap().as_str().to_string(),
            alt: cap.get(1).unwrap().as_str().to_string(),
            lines: m.as_str().chars().filter(|c| *c == '\n').count(),
        });
        last_end = m.end();
    }
    let after = &text[last_end..];
    if !after.trim().is_empty() {
        temp.push(PreviewSection::Markdown(after.to_string()));
    }
}
