import os

with open('crates/katana-ui/src/http_cache_loader.rs', 'r') as f:
    text = f.read()

# Extract the tests module
start = text.find('#[cfg(test)]\nmod tests {')
if start != -1:
    tests_content = text[start:]
    
    # We replace "mod tests {" to include correct imports
    new_tests_content = tests_content.replace(
        "mod tests {\n    use super::*;",
        "mod tests {\n    use super::*;\n    use egui::load::BytesPoll;\n    use super::types::{CachedFile, HttpCacheEntry};\n    use super::disk::{cache_key, write_cached_file_for_uri, read_cached_file, remove_cache_file, CACHE_BODY_EXTENSION, CACHE_META_EXTENSION};\n    use super::fetch::{process_fetch_response, entry_to_bytes_result};\n    use egui::Context;\n    use std::task::Poll;\n    use std::sync::Arc;"
    )

    with open('crates/katana-ui/src/http_cache_loader/tests.rs', 'w') as f:
        f.write("#![cfg(test)]\n")
        f.write(new_tests_content)
