// Build script for macOS native menu bar.
// Compiles and links the Objective-C file (macos_menu.m).

fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/macos_menu.m")
            .flag("-fobjc-arc")
            .compile("macos_menu");

        println!("cargo:rustc-link-lib=framework=Cocoa");
    }
}
