use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

fn main() {
    let locales_dir = PathBuf::from("crates/katana-ui/locales");
    let mut locales = HashMap::new();

    for entry in fs::read_dir(locales_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if filename == "languages.json" {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&content).unwrap();
            locales.insert(filename, json);
        }
    }

    let mut overlap_values = HashSet::new();

    fn get_leaves(val: &serde_json::Value, path: &str, leaves: &mut HashMap<String, String>) {
        if path.contains(".key") && path.starts_with("settings.tabs[") {
            return;
        }
        if path == "error.render_error" || path == "terms.version_label" {
            return;
        }
        match val {
            serde_json::Value::String(s) => {
                leaves.insert(path.to_string(), s.trim().to_string());
            }
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let new_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    get_leaves(v, &new_path, leaves);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, i);
                    get_leaves(v, &new_path, leaves);
                }
            }
            _ => {}
        }
    }

    let mut all_leaves = HashMap::new();
    for (filename, json) in &locales {
        let mut leaves = HashMap::new();
        get_leaves(json, "", &mut leaves);
        all_leaves.insert(filename.clone(), leaves);
    }

    let filenames: Vec<String> = locales.keys().cloned().collect();

    for i in 0..filenames.len() {
        for j in (i + 1)..filenames.len() {
            let f1 = &filenames[i];
            let f2 = &filenames[j];

            let leaves1 = all_leaves.get(f1).unwrap();
            let leaves2 = all_leaves.get(f2).unwrap();

            for (path, val1) in leaves1 {
                if let Some(val2) = leaves2.get(path) {
                    if val1 == val2 {
                        overlap_values.insert(val1.clone());
                        println!("Overlap [{} vs {}] at {}: {}", f1, f2, path, val1);
                    }
                }
            }
        }
    }

    println!(
        "\nNumber of overlapping exact values: {}",
        overlap_values.len()
    );
    let mut sorted_overlaps: Vec<_> = overlap_values.into_iter().collect();
    sorted_overlaps.sort();
    println!("ALLOWED_OVERLAPS: {:#?}", sorted_overlaps);
}