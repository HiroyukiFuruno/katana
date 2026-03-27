use std::fs;

fn main() {
    let code = fs::read_to_string("crates/katana-ui/src/settings_window.rs").unwrap();
    let ast = syn::parse_file(&code).unwrap();
    // we want to know what syn extracts
    println!("Parsed!");
}
