use ehttp::Request;

// struct GithubRelease removed since we parse HTML now

/// Helper function to compare semantic versions cleanly.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_parts: Vec<&str> = current.trim_start_matches('v').split('.').collect();
    let latest_parts: Vec<&str> = latest.trim_start_matches('v').split('.').collect();

    let len = std::cmp::max(current_parts.len(), latest_parts.len());
    for i in 0..len {
        let curr = current_parts
            .get(i)
            .unwrap_or(&"0")
            .parse::<u32>()
            .unwrap_or(0);
        let lat = latest_parts
            .get(i)
            .unwrap_or(&"0")
            .parse::<u32>()
            .unwrap_or(0);
        if lat > curr {
            return true;
        } else if lat < curr {
            return false;
        }
    }
    false
}

/// Parses the `ehttp` response to extract the version.
pub fn parse_update_response(result: Result<ehttp::Response, String>) -> Result<String, String> {
    match result {
        Ok(response) => {
            if response.ok {
                if let Some(text) = response.text() {
                    if let Some(latest_version) = extract_version_from_html(text) {
                        return Ok(latest_version);
                    }
                }
                Err("Failed to parse GitHub HTML response".to_string())
            } else {
                Err(format!("GitHub HTML error: {}", response.status))
            }
        }
        Err(e) => Err(e),
    }
}

/// Asynchronously checks for the latest release on GitHub.
/// The `on_done` callback is invoked with `Ok(latest_version)` or `Err(reason)`.
pub fn check_for_updates<F>(on_done: F)
where
    F: FnOnce(Result<String, String>) + Send + 'static,
{
    let url = "https://github.com/HiroyukiFuruno/KatanA/releases/latest";
    let req = Request::get(url);

    ehttp::fetch(req, move |result| {
        on_done(parse_update_response(result));
    });
}

/// Extracts the version string from GitHub Releases HTML.
pub fn extract_version_from_html(html: &str) -> Option<String> {
    // We look for the breadcrumb or tag link: href="/HiroyukiFuruno/KatanA/releases/tag/vX.Y.Z"
    let token = "href=\"/HiroyukiFuruno/KatanA/releases/tag/";
    if let Some(start_idx) = html.find(token) {
        let substr = &html[start_idx + token.len()..];
        if let Some(end_idx) = substr.find('"') {
            let version_text = substr[..end_idx].trim();
            let latest_version = version_text.trim_start_matches('v').to_string();
            return Some(latest_version);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("v0.3.0", "v0.3.1"));
        assert!(is_newer_version("0.3.0", "0.3.1"));
        assert!(is_newer_version("v0.3.1", "v0.4.0"));
        assert!(is_newer_version("v0.3.1", "v1.0.0"));

        assert!(!is_newer_version("v0.3.1", "v0.3.1"));
        assert!(!is_newer_version("v0.3.1", "v0.3.0"));
        assert!(!is_newer_version("v1.0.0", "v0.9.9"));

        // Edge cases
        assert!(is_newer_version("v0.3", "v0.3.1"));
        assert!(!is_newer_version("v0.3.1", "v0.3"));
    }

    #[test]
    fn test_extract_version_from_html_success() {
        let sample_html = r#"
            <li class="breadcrumb-item"><a href="/HiroyukiFuruno/KatanA/releases/tag/v0.3.1">v0.3.1</a></li>
        "#;
        assert_eq!(
            extract_version_from_html(sample_html),
            Some("0.3.1".to_string())
        );
    }

    #[test]
    fn test_extract_version_from_html_failure() {
        let bad_html = r#"
            <div>No version here</div>
        "#;
        assert_eq!(extract_version_from_html(bad_html), None);
    }

    #[test]
    fn test_parse_update_response_success() {
        let html = r#"<a href="/HiroyukiFuruno/KatanA/releases/tag/v1.2.3">"#;
        let response = ehttp::Response {
            url: "".to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            headers: ehttp::Headers::new(&[]),
            bytes: html.as_bytes().to_vec(),
        };
        assert_eq!(parse_update_response(Ok(response)), Ok("1.2.3".to_string()));
    }

    #[test]
    fn test_parse_update_response_missing_html() {
        let response = ehttp::Response {
            url: "".to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            headers: ehttp::Headers::new(&[]),
            bytes: vec![], // Empty or no HTML match
        };
        assert_eq!(
            parse_update_response(Ok(response)),
            Err("Failed to parse GitHub HTML response".to_string())
        );
    }

    #[test]
    fn test_parse_update_response_http_error() {
        let response = ehttp::Response {
            url: "".to_string(),
            ok: false,
            status: 403,
            status_text: "Forbidden".to_string(),
            headers: ehttp::Headers::new(&[]),
            bytes: vec![],
        };
        assert_eq!(
            parse_update_response(Ok(response)),
            Err("GitHub HTML error: 403".to_string())
        );
    }

    #[test]
    fn test_parse_update_response_network_error() {
        assert_eq!(
            parse_update_response(Err("Network timeout".to_string())),
            Err("Network timeout".to_string())
        );
    }
}
