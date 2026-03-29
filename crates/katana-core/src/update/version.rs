use serde::Deserialize;

/// Information about a new release fetched from the upstream repository.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReleaseInfo {
    /// The tag name of the release, e.g., "v0.7.0".
    pub tag_name: String,
    /// The URL to view the release in a browser.
    pub html_url: String,
    /// The body or description of the release.
    pub body: String,
    /// The direct download URL for the update package.
    pub download_url: String,
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

fn build_release_info(tag_name: String) -> ReleaseInfo {
    let html_url = format!(
        "https://github.com/HiroyukiFuruno/KatanA/releases/tag/{}",
        tag_name
    );
    let download_url = format!(
        "https://github.com/HiroyukiFuruno/KatanA/releases/download/{}/KatanA-macOS.zip",
        tag_name
    );
    let body = format!("### 🚀 New version {} is available\n\nPlease check the [GitHub Releases page]({}) for detailed changes and release notes.\nClick \"Install and Restart\" to automatically apply the update.", &tag_name, html_url);
    ReleaseInfo {
        tag_name,
        html_url,
        body,
        download_url,
    }
}

/// Checks the upstream API for a newer version than the `current_version`.
pub fn check_for_updates(
    current_version: &str,
    api_url_override: Option<&str>,
) -> anyhow::Result<Option<ReleaseInfo>> {
    let check_url =
        api_url_override.unwrap_or("https://github.com/HiroyukiFuruno/KatanA/releases/latest");
    let response = ureq::get(check_url)
        .set("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
        .call()?;

    let final_url = response.get_url();
    let tag_name = final_url
        .split('/')
        .next_back()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse release tag ({})", final_url))?
        .to_string();

    if is_newer_version(current_version, &tag_name) {
        Ok(Some(build_release_info(tag_name)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.6.4", "v0.7.0"));
        assert!(!is_newer_version("v0.6.4", "v0.6.4"));
        assert!(!is_newer_version("0.7.0", "v0.6.4"));
    }

    #[test]
    fn test_check_for_updates_network_success() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);
        let check_url = format!("{}/releases/latest", base_url);

        thread::spawn(move || {
            use std::io::Read;
            for _ in 0..2 {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = std::str::from_utf8(&buf[..n]).unwrap_or("");

                if request.contains("GET /releases/latest") {
                    let location = format!("http://127.0.0.1:{}/releases/tag/v0.7.0", port);
                    let _ = stream.write_all(
                            format!(
                                "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                                location
                            )
                            .as_bytes(),
                        );
                } else {
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                    );
                }
            }
        });

        let res = check_for_updates("0.6.4", Some(&check_url)).expect("Network check failed");
        let info = res.expect("Should contain newer version info");
        assert_eq!(info.tag_name, "v0.7.0");
    }

    #[test]
    fn test_check_for_updates_network_error() {
        let res = check_for_updates("0.6.4", Some("http://127.0.0.1:0/impossible"));
        assert!(res.is_err());
    }
}
