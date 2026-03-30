
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        println!("cargo:rerun-if-changed=src/macos_appearance.m");
        cc::Build::new()
            .file("src/macos_appearance.m")
            .flag("-fobjc-arc")
            .compile("macos_appearance");
        println!("cargo:rustc-link-lib=framework=Cocoa");
    }
}