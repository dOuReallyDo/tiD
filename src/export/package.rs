//! Package builder — Full-package ZIP export.

use crate::paths;
use std::io::Write;
use zip::write::SimpleFileOptions;

/// Build a full-package ZIP containing exe + frontend + data template.
pub fn build_full_package() -> anyhow::Result<std::path::PathBuf> {
    let exports = paths::exports_dir();
    std::fs::create_dir_all(&exports)?;

    let zip_path = exports.join(format!("tiD-package-{}.zip",
        chrono::Utc::now().format("%Y%m%dT%H%M%S")));

    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let base = paths::base_dir();

    // Add current executable (best-effort)
    if let Ok(exe_path) = std::env::current_exe() {
        if exe_path.exists() {
            add_file_to_zip(&mut zip, &exe_path, "tiD/tiD.exe", &options)?;
        }
    }

    // Add frontend/dist/
    let frontend = paths::frontend_dir();
    if frontend.exists() {
        add_dir_to_zip(&mut zip, &frontend, "tiD/frontend/dist/", &options)?;
    }

    // Add data/sources/ template
    let sources = paths::sources_dir();
    if sources.exists() {
        add_dir_to_zip(&mut zip, &sources, "tiD/data/sources/", &options)?;
    }

    // Add START_tiD.bat
    let bat = base.join("START_tiD.bat");
    if bat.exists() {
        add_file_to_zip(&mut zip, &bat, "tiD/START_tiD.bat", &options)?;
    }

    // Add README.md
    let readme = base.join("README.md");
    if readme.exists() {
        add_file_to_zip(&mut zip, &readme, "tiD/README.md", &options)?;
    }

    zip.finish()?;

    Ok(zip_path)
}

fn add_file_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    path: &std::path::Path,
    entry_name: &str,
    options: &SimpleFileOptions,
) -> anyhow::Result<()> {
    let data = std::fs::read(path)?;
    zip.start_file(entry_name, *options)?;
    zip.write_all(&data)?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &std::path::Path,
    prefix: &str,
    options: &SimpleFileOptions,
) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let entry_name = format!("{}{}", prefix, name);

        if path.is_dir() {
            add_dir_to_zip(zip, &path, &format!("{}{}/", prefix, name), options)?;
        } else {
            add_file_to_zip(zip, &path, &entry_name, options)?;
        }
    }
    Ok(())
}