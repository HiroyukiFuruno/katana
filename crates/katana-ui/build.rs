// Build script for macOS native menu bar and compile-time metadata.
// Compiles and links the Objective-C file (macos_menu.m).
// Captures the rustc version for display in the About dialog.

fn main() {
    // Tell Cargo to allow #[cfg(coverage)] to satisfy `unexpected_cfgs` lint when testing
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    // Capture rustc version (e.g. "rustc 1.82.0 (f6e511eec 2024-10-15)")
    // and expose it as KATANA_RUSTC_VERSION for use with env!() in about_info.rs.
    if let Ok(output) = std::process::Command::new("rustc")
        .arg("--version")
        .output()
    {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=KATANA_RUSTC_VERSION={version}");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        println!("cargo:rerun-if-changed=src/macos_menu.m");
        println!("cargo:rerun-if-changed=Info.plist");

        cc::Build::new()
            .file("src/macos_menu.m")
            .flag("-fobjc-arc")
            .compile("macos_menu");

        println!("cargo:rustc-link-lib=framework=Cocoa");
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{}/Info.plist",
            manifest_dir
        );
    }
}
