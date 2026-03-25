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

/// Represents a fully prepared update that is ready to be executed.
#[derive(Debug)]
pub struct UpdatePreparation {
    pub temp_dir: tempfile::TempDir,
    pub script_path: std::path::PathBuf,
}

/// Prepares the update by downloading, extracting, and generating the relauncher script.
pub fn prepare_update(
    download_url: &str,
    target_app_path: &std::path::Path,
) -> anyhow::Result<UpdatePreparation> {
    let temp_dir = tempfile::tempdir()?;

    let zip_path = temp_dir.path().join("update.zip");
    download_update(download_url, &zip_path)?;

    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)?;
    extract_update(&zip_path, &extract_dir)?;

    // Find the .app bundle in the extracted directory
    // Typically it's "KatanA.app" inside the root of the zip.
    let app_name = target_app_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("KatanA.app"));

    let extracted_app_path = extract_dir.join(app_name);
    if !extracted_app_path.exists() {
        anyhow::bail!("Extracted update does not contain the expected application bundle");
    }

    let script_path = temp_dir.path().join("relauncher.sh");
    let target = target_app_path;
    let extracted = &extracted_app_path;
    let temp = temp_dir.path();
    generate_relauncher_script(extracted, target, &script_path, temp)?;

    Ok(UpdatePreparation {
        temp_dir,
        script_path,
    })
}

/// Executes the background relauncher and exits the current process.
#[cfg(not(test))]
#[cfg(not(coverage))]
pub fn execute_relauncher(prep: UpdatePreparation) -> anyhow::Result<()> {
    // Consume the temp dir to prevent its automatic deletion
    #[allow(deprecated)]
    let _temp_path = prep.temp_dir.into_path();

    std::process::Command::new(&prep.script_path).spawn()?;
    std::process::exit(0);
}

/// Generates the bash script used for atomic replacement and quarantine removal.
pub fn generate_relauncher_script(
    extracted_app: &std::path::Path,
    target_app: &std::path::Path,
    script_path: &std::path::Path,
    temp_dir_path: &std::path::Path,
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
rm -rf "{temp_dir}"
"#,
        target = target_app.display(),
        extracted = extracted_app.display(),
        temp_dir = temp_dir_path.display()
    );

    std::fs::write(script_path, content)?;

    const RELAUNCHER_SCRIPT_PERMISSIONS: u32 = 0o755;

    // Make the script executable
    let mut perms = std::fs::metadata(script_path)?.permissions();
    perms.set_mode(RELAUNCHER_SCRIPT_PERMISSIONS);
    std::fs::set_permissions(script_path, perms)?;

    Ok(())
}

#[derive(Debug, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpdateAvailable(ReleaseInfo),
    Downloading,
    ReadyToRestart(UpdatePreparation),
    Error(String),
}

pub struct UpdateManager {
    pub current_version: String,
    pub api_url_override: Option<String>,
    pub target_app_path: std::path::PathBuf,
    pub state: UpdateState,
    pub last_checked: Option<std::time::Instant>,
    pub check_interval: std::time::Duration,
}

impl UpdateManager {
    pub fn new(current_version: String, target_app_path: std::path::PathBuf) -> Self {
        const DEFAULT_CHECK_INTERVAL_SECS: u64 = 86_400;
        Self {
            current_version,
            api_url_override: None,
            target_app_path,
            state: UpdateState::Idle,
            last_checked: None,
            check_interval: std::time::Duration::from_secs(DEFAULT_CHECK_INTERVAL_SECS),
        }
    }

    pub fn should_check_for_updates(&self) -> bool {
        match self.last_checked {
            Some(last) => last.elapsed() >= self.check_interval,
            None => true,
        }
    }

    pub fn set_api_url_override(&mut self, url: String) {
        self.api_url_override = Some(url);
    }

    pub fn set_check_interval(&mut self, interval: std::time::Duration) {
        self.check_interval = interval;
    }

    pub fn transition_to(&mut self, new_state: UpdateState) {
        if matches!(new_state, UpdateState::Checking) {
            self.last_checked = Some(std::time::Instant::now());
        }
        self.state = new_state;
    }
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

        generate_relauncher_script(&extracted_path, &target_path, &script_path, temp_dir.path())
            .unwrap();

        assert!(script_path.exists());

        let content = std::fs::read_to_string(&script_path).unwrap();
        assert!(content.contains(&format!("rm -rf \"{}\"", target_path.display())));
        assert!(content.contains(&format!(
            "mv \"{}\" \"{}\"",
            extracted_path.display(),
            target_path.display()
        )));
        assert!(content.contains(&format!("xattr -cr \"{}\"", target_path.display())));
        assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));

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

    #[test]
    fn test_prepare_update() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;
        use zip::write::SimpleFileOptions;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/update.zip", port);

        thread::spawn(move || {
            use std::io::Read;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0; 1024];
                let _ = stream.read(&mut buf);

                let mut zip_buf = Vec::new();
                {
                    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
                    let options = SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored);
                    zip.add_directory("KatanA.app/", options).unwrap();
                    zip.finish().unwrap();
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    zip_buf.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&zip_buf);
            }
        });

        let target_app = std::path::Path::new("/Applications/KatanA.app");
        let prep = prepare_update(&url, target_app).expect("prepare_update should succeed");

        assert!(prep.script_path.exists());
        let content = std::fs::read_to_string(&prep.script_path).unwrap();
        assert!(content.contains(&format!("rm -rf \"{}\"", prep.temp_dir.path().display())));
    }

    #[test]
    fn test_prepare_update_missing_app() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;
        use zip::write::SimpleFileOptions;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/broken.zip", port);

        thread::spawn(move || {
            use std::io::Read;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0; 1024];
                let _ = stream.read(&mut buf);

                let mut zip_buf = Vec::new();
                {
                    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
                    let options = SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored);
                    zip.add_directory("Wrong.app/", options).unwrap();
                    zip.finish().unwrap();
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    zip_buf.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&zip_buf);
            }
        });

        let target_app = std::path::Path::new("/Applications/KatanA.app");
        let res = prepare_update(&url, target_app);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Extracted update does not contain the expected application bundle"
        );
    }

    #[test]
    fn test_update_manager_and_state() {
        let target = std::path::PathBuf::from("/Applications/KatanA.app");
        let mut manager = UpdateManager::new("0.6.4".to_string(), target.clone());

        assert_eq!(manager.current_version, "0.6.4");
        assert_eq!(manager.target_app_path, target);
        assert!(matches!(manager.state, UpdateState::Idle));

        // Default interval is 24 hours. Because last_checked is None, it should check.
        assert!(manager.should_check_for_updates());

        // Test API override
        manager.set_api_url_override("http://localhost".to_string());
        assert_eq!(
            manager.api_url_override.as_deref(),
            Some("http://localhost")
        );

        // Test interval override
        manager.set_check_interval(std::time::Duration::from_secs(3600));
        assert_eq!(manager.check_interval, std::time::Duration::from_secs(3600));

        // State transition to Checking records the last_checked time
        manager.transition_to(UpdateState::Checking);
        assert!(matches!(manager.state, UpdateState::Checking));
        assert!(manager.last_checked.is_some());

        // Immediately checking again should be false (because 1 hour hasn't passed)
        assert!(!manager.should_check_for_updates());

        // Fast forward by faking last_checked time
        manager.last_checked =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(4000));
        assert!(manager.should_check_for_updates());

        // Test another transition
        manager.transition_to(UpdateState::Error("dummy error".to_string()));
        assert!(matches!(manager.state, UpdateState::Error(_)));

        // Ensure default works
        let default_state = UpdateState::default();
        assert!(matches!(default_state, UpdateState::Idle));
    }
}
