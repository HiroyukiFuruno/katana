use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub body: String,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProgress {
    Downloading { downloaded: u64, total: Option<u64> },
    Extracting { current: usize, total: usize },
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

pub fn check_for_updates(
    current_version: &str,
    api_url_override: Option<&str>,
) -> anyhow::Result<Option<ReleaseInfo>> {
    let check_url =
        api_url_override.unwrap_or("https://github.com/HiroyukiFuruno/KatanA/releases/latest");

    // Do not use the API. Send a direct HTTP GET request to the latest endpoint.
    // ureq securely follows the 302 redirect by default, landing at the tagged release.
    let response = ureq::get(check_url)
        .set("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
        .call()?;

    let final_url = response.get_url();

    let tag_name = final_url
        .split('/')
        .next_back()
        .ok_or_else(|| anyhow::anyhow!("リリースタグの解析に失敗しました ({})", final_url))?
        .to_string();

    if is_newer_version(current_version, &tag_name) {
        let html_url = format!(
            "https://github.com/HiroyukiFuruno/KatanA/releases/tag/{}",
            tag_name
        );
        let download_url = format!(
            "https://github.com/HiroyukiFuruno/KatanA/releases/download/{}/KatanA-macOS.zip",
            tag_name
        );

        let body = format!(
            "### 🚀 最新バージョン {} が利用可能です\n\n詳しい変更内容やリリースノートについては、[GitHub Releases ページ]({}) をご確認ください。\n自動アップデートを実行するには「インストールして再起動」をクリックしてください。",
            &tag_name, html_url
        );

        Ok(Some(ReleaseInfo {
            tag_name,
            html_url,
            body,
            download_url,
        }))
    } else {
        Ok(None)
    }
}

/// Downloads a file from the given URL to the destination path.
pub fn download_update<F>(
    url: &str,
    dest_path: &std::path::Path,
    mut on_progress: F,
) -> anyhow::Result<()>
where
    F: FnMut(u64, Option<u64>),
{
    use std::io::{Read, Write};
    let response = ureq::get(url)
        .set("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
        .call()?;

    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse().ok());
    let mut reader = response.into_reader();
    let mut out_file = std::fs::File::create(dest_path)?;

    const DOWNLOAD_BUFFER_SIZE: usize = 65536;
    let mut buffer = [0; DOWNLOAD_BUFFER_SIZE]; // 64KB buffer
    let mut downloaded = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        out_file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        on_progress(downloaded, total_size);
    }

    Ok(())
}

/// Extracts a ZIP archive into the destination directory.
pub fn extract_update<F>(
    zip_path: &std::path::Path,
    extract_to_dir: &std::path::Path,
    mut on_progress: F,
) -> anyhow::Result<()>
where
    F: FnMut(usize, usize),
{
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let total_files = archive.len();

    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }

        // Notify progress (1-indexed)
        on_progress(i + 1, total_files);
    }

    Ok(())
}

/// Represents a fully prepared update that is ready to be executed.
#[derive(Debug)]
pub struct UpdatePreparation {
    pub temp_dir: tempfile::TempDir,
    pub script_path: std::path::PathBuf,
}

/// Prepares the update by downloading, extracting, and generating the relauncher script.
pub fn prepare_update<F>(
    download_url: &str,
    target_app_path: &std::path::Path,
    mut on_progress: F,
) -> anyhow::Result<UpdatePreparation>
where
    F: FnMut(UpdateProgress),
{
    let temp_dir = tempfile::tempdir()?;

    let zip_path = temp_dir.path().join("update.zip");
    download_update(download_url, &zip_path, |downloaded, total| {
        on_progress(UpdateProgress::Downloading { downloaded, total });
    })?;

    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)?;
    extract_update(&zip_path, &extract_dir, |current, total| {
        on_progress(UpdateProgress::Extracting { current, total });
    })?;

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
    fn test_check_for_updates_network_success() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        // Mock server: first request gets a 302 redirect to /releases/tag/v0.7.0,
        // second request (after redirect) gets a 200 OK.
        // ureq follows the redirect and get_url() returns the final URL,
        // from which we extract "v0.7.0".
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);
        let check_url = format!("{}/releases/latest", base_url);

        thread::spawn(move || {
            use std::io::Read;
            // Handle up to 2 connections: initial request + redirect follow-up
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 2048];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let request = std::str::from_utf8(&buf[..n]).unwrap_or("");

                    if request.contains("GET /releases/latest") {
                        // Return a 302 redirect to the tagged release page
                        let location = format!("http://127.0.0.1:{}/releases/tag/v0.7.0", port);
                        let _ = stream.write_all(
                            format!(
                                "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                                location
                            )
                            .as_bytes(),
                        );
                    } else {
                        // Follow-up GET /releases/tag/v0.7.0 — return minimal HTML
                        let _ = stream.write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                        );
                    }
                }
            }
        });

        let res = check_for_updates("0.6.4", Some(&check_url)).expect("Network check failed");
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

        download_update(&url, &dest, |_, _| {}).unwrap();
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
            zip.add_directory("somedir/", options.clone()).unwrap();
            // Create a file with a relative path resolving outside the root, triggering `enclosed_name() == None`
            zip.start_file("../outside.txt", options).unwrap();
            zip.write_all(b"Should be skipped").unwrap();
            zip.finish().unwrap();
        }

        extract_update(&zip_path, &extract_dir, |_, _| {}).unwrap();

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
        let prep = prepare_update(&url, target_app, |_| {}).expect("prepare_update should succeed");

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
        let res = prepare_update(&url, target_app, |_| {});
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
