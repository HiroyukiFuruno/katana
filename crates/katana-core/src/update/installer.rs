use crate::update::download::{download_update, extract_update};
use crate::update::UpdateProgress;

/// Represents a fully prepared update that is ready to be executed.
#[derive(Debug)]
pub struct UpdatePreparation {
    /// The temporary directory holding the extracted update.
    pub temp_dir: tempfile::TempDir,
    /// The path to the newly extracted KatanA app bundle.
    pub app_bundle_path: std::path::PathBuf,
    /// The path to the custom bash relauncher script.
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
    download_update(download_url, &zip_path, &mut on_progress)?;

    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)?;
    extract_update(&zip_path, &extract_dir, &mut on_progress)?;

    let app_name = target_app_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("KatanA.app"));
    let extracted_app_path = extract_dir.join(app_name);
    if !extracted_app_path.exists() {
        anyhow::bail!("Extracted update does not contain the expected application bundle");
    }

    let script_path = temp_dir.path().join("relauncher.sh");
    generate_relauncher_script(
        &extracted_app_path,
        target_app_path,
        &script_path,
        temp_dir.path(),
    )?;

    Ok(UpdatePreparation {
        temp_dir,
        app_bundle_path: extracted_app_path,
        script_path,
    })
}

/// Executes the background relauncher and exits the current process.
#[cfg(not(test))]
#[cfg(not(coverage))]
pub fn execute_relauncher(prep: UpdatePreparation) -> anyhow::Result<()> {
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

    let content = generate_script_content(target_app, extracted_app, temp_dir_path);
    std::fs::write(script_path, content)?;

    const RELAUNCHER_SCRIPT_PERMISSIONS: u32 = 0o755;

    let mut perms = std::fs::metadata(script_path)?.permissions();
    perms.set_mode(RELAUNCHER_SCRIPT_PERMISSIONS);
    std::fs::set_permissions(script_path, perms)?;

    Ok(())
}

fn generate_script_content(
    target_app: &std::path::Path,
    extracted_app: &std::path::Path,
    temp_dir_path: &std::path::Path,
) -> String {
    format!(
        r#"#!/bin/bash
set -e
sleep 1
if command -v brew >/dev/null 2>&1; then
    if brew list --cask | grep -q "^katana-desktop$"; then
        echo "Removing KatanA from Homebrew management..."
        brew uninstall --cask katana-desktop --force || true
        brew untap HiroyukiFuruno/katana || true
    fi
fi
rm -rf "{target}"
mv "{extracted}" "{target}"
xattr -cr "{target}"
open "{target}"
rm -rf "{temp_dir}"
"#,
        target = target_app.display(),
        extracted = extracted_app.display(),
        temp_dir = temp_dir_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(content.contains("brew uninstall --cask katana-desktop --force"));
        assert!(content.contains(&format!("xattr -cr \"{}\"", target_path.display())));
        assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));

        let perms = std::fs::metadata(&script_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111, "Script must be executable");
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
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf);

            let mut zip_buf = Vec::new();
            {
                let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
                let options =
                    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
                zip.add_directory("KatanA.app/", options).unwrap();
                zip.finish().unwrap();
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                zip_buf.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(&zip_buf);
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
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf);

            let mut zip_buf = Vec::new();
            {
                let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
                let options =
                    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
                zip.add_directory("Wrong.app/", options).unwrap();
                zip.finish().unwrap();
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                zip_buf.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(&zip_buf);
        });

        let target_app = std::path::Path::new("/Applications/KatanA.app");
        let res = prepare_update(&url, target_app, |_| {});
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Extracted update does not contain the expected application bundle"
        );
    }
}
