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
}
