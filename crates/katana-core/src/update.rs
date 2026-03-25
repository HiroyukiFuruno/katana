use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub body: String,
    pub download_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<GithubReleaseAsset>,
}

/// Parses the semver versions (stripping 'v' prefix if present)
/// and returns true if `upstream` is strictly greater than `current`.
pub fn is_newer_version(current: &str, upstream: &str) -> bool {
    let current_clean = current.strip_prefix('v').unwrap_or(current);
    let upstream_clean = upstream.strip_prefix('v').unwrap_or(upstream);

    if let (Ok(curr_ver), Ok(up_ver)) = (
        semver::Version::parse(current_clean),
        semver::Version::parse(upstream_clean),
    ) {
        up_ver > curr_ver
    } else {
        false
    }
}

pub fn parse_release_response(
    current_version: &str,
    json_str: &str,
) -> anyhow::Result<Option<ReleaseInfo>> {
    let release: GithubRelease = serde_json::from_str(json_str)?;

    if is_newer_version(current_version, &release.tag_name) {
        let download_url = release
            .assets
            .into_iter()
            .find(|a| a.name.ends_with(".zip") || a.name.contains("macOS"))
            .map(|a| a.browser_download_url)
            .unwrap_or_else(|| release.html_url.clone());

        Ok(Some(ReleaseInfo {
            tag_name: release.tag_name,
            html_url: release.html_url.clone(),
            body: release.body.unwrap_or_default(),
            download_url,
        }))
    } else {
        Ok(None)
    }
}

pub fn check_for_updates(
    current_version: &str,
    api_url_override: Option<&str>,
) -> anyhow::Result<Option<ReleaseInfo>> {
    let api_url = api_url_override
        .unwrap_or("https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest");

    let resp_string = ureq::get(api_url)
        .set("Accept", "application/vnd.github.v3+json")
        .set("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
        .call()?
        .into_string()?;

    parse_release_response(current_version, &resp_string)
}

/// Downloads a file from the given URL to the destination path.
pub fn download_update(url: &str, dest_path: &std::path::Path) -> anyhow::Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
        .call()?;
    let mut reader = response.into_reader();
    let mut out_file = std::fs::File::create(dest_path)?;
    std::io::copy(&mut reader, &mut out_file)?;
    Ok(())
}

/// Extracts a ZIP archive into the destination directory.
pub fn extract_update(
    zip_path: &std::path::Path,
    extract_to_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(extract_to_dir)?;
    Ok(())
}

/// Generates the bash script used for atomic replacement and quarantine removal.
pub fn generate_relauncher_script(
    extracted_app: &std::path::Path,
    target_app: &std::path::Path,
    script_path: &std::path::Path,
) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let content = format!(
        r#"#!/bin/bash
# KatanA Auto-Update Relauncher Script
set -e

# Wait a brief moment to ensure the old process has fully exited
sleep 1

echo "Replacing application..."
rm -rf "{target}"
mv "{extracted}" "{target}"

echo "Removing Gatekeeper quarantine attributes..."
xattr -cr "{target}"

echo "Relaunching application..."
open "{target}"

echo "Cleaning up..."
rm -f "$0"
"#,
        target = target_app.display(),
        extracted = extracted_app.display()
    );

    std::fs::write(script_path, content)?;

    const RELAUNCHER_SCRIPT_PERMISSIONS: u32 = 0o755;

    // Make the script executable
    let mut perms = std::fs::metadata(script_path)?.permissions();
    perms.set_mode(RELAUNCHER_SCRIPT_PERMISSIONS);
    std::fs::set_permissions(script_path, perms)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert_eq!(is_newer_version("0.6.4", "v0.7.0"), true);
        assert_eq!(is_newer_version("v0.6.4", "v0.6.4"), false);
        assert_eq!(is_newer_version("0.7.0", "v0.6.4"), false);
    }

    #[test]
    fn test_parse_release_response_newer() {
        let json = r#"{
            "tag_name": "v0.7.0",
            "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.7.0",
            "body": "Release notes here",
            "assets": [
                {
                    "name": "KatanA-macOS.zip",
                    "browser_download_url": "https://github.com/.../KatanA-macOS.zip"
                }
            ]
        }"#;

        let result = parse_release_response("0.6.4", json).unwrap().unwrap();
        assert_eq!(result.tag_name, "v0.7.0");
        assert_eq!(
            result.download_url,
            "https://github.com/.../KatanA-macOS.zip"
        );
        assert_eq!(result.body, "Release notes here");
    }

    #[test]
    fn test_parse_release_response_older() {
        let json = r#"{ "tag_name": "v0.6.0", "html_url": "...", "body": null, "assets": [] }"#;
        let result = parse_release_response("0.6.4", json).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_check_for_updates_network_success() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/latest.json", port);

        // Spawn a tiny server that just replies with a 200 OK + valid JSON
        thread::spawn(move || {
            use std::io::Read;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0; 1024];
                let _ = stream.read(&mut buf); // Consume the request headers to prevent TCP RST

                let body = r#"{
                    "tag_name": "v0.7.0",
                    "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.7.0",
                    "assets": []
                }"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let res = check_for_updates("0.6.4", Some(&url)).expect("Network check failed");
        let info = res.expect("Should contain newer version info");
        assert_eq!(info.tag_name, "v0.7.0");
    }

    #[test]
    fn test_check_for_updates_network_error() {
        // Point to an unbindable/closed local port
        let res = check_for_updates("0.6.4", Some("http://127.0.0.1:0/impossible"));
        assert!(res.is_err());
    }

    #[test]
    fn test_generate_relauncher_script() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let extracted_path = temp_dir.path().join("KatanA-extract.app");
        let target_path = temp_dir.path().join("KatanA.app");
        let script_path = temp_dir.path().join("relauncher.sh");

        generate_relauncher_script(&extracted_path, &target_path, &script_path).unwrap();

        assert!(script_path.exists());

        let content = std::fs::read_to_string(&script_path).unwrap();
        assert!(content.contains(&format!("rm -rf \"{}\"", target_path.display())));
        assert!(content.contains(&format!(
            "mv \"{}\" \"{}\"",
            extracted_path.display(),
            target_path.display()
        )));
        assert!(content.contains(&format!("xattr -cr \"{}\"", target_path.display())));

        let perms = std::fs::metadata(&script_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111, "Script must be executable");
    }

    #[test]
    fn test_download_update() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/update.zip", port);

        thread::spawn(move || {
            use std::io::Read;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0; 1024];
                let _ = stream.read(&mut buf);

                let body = b"mock zip payload";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(body);
            }
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("update.zip");

        download_update(&url, &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"mock zip payload");
    }

    #[test]
    fn test_extract_update() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extracted");

        // Create a dummy zip file
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("hello.txt", options.clone()).unwrap();
            zip.write_all(b"Hello from ZIP").unwrap();
            zip.add_directory("somedir/", options).unwrap();
            zip.finish().unwrap();
        }

        extract_update(&zip_path, &extract_dir).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(std::fs::read(&extracted_file).unwrap(), b"Hello from ZIP");
        assert!(extract_dir.join("somedir").is_dir());
    }
}
