
fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    if let Ok(output) = std::process::Command::new("rustc")
        .arg("--version")
        .output()
    {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=KATANA_RUSTC_VERSION={version}");
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "dev".to_string());
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=KATANA_BUILD={}-{}", profile, hash);
        } else {
            println!("cargo:rustc-env=KATANA_BUILD={}", profile);
        }
    } else {
        println!("cargo:rustc-env=KATANA_BUILD={}", profile);
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