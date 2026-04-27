//! Path resolution — mirrors tiC's paths.py
//!
//! Layout:
//!   data/sources/  — factory originals
//!   data/inputs/   — user-uploaded files (override sources)
//!   data/exports/  — latest exports
//!   data/archive/  — historical exports

use std::path::PathBuf;

pub const PORT: u16 = 5002;

/// Base directory — where the executable lives.
pub fn base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn data_dir() -> PathBuf {
    base_dir().join("data")
}

pub fn sources_dir() -> PathBuf {
    data_dir().join("sources")
}

pub fn inputs_dir() -> PathBuf {
    data_dir().join("inputs")
}

pub fn exports_dir() -> PathBuf {
    data_dir().join("exports")
}

pub fn archive_dir() -> PathBuf {
    data_dir().join("archive")
}

pub fn frontend_dir() -> PathBuf {
    base_dir().join("frontend").join("dist")
}

/// Resolve input file path: prefer data/inputs/, fall back to data/sources/.
/// Returns Err if file not found in either location.
pub fn resolve_input(filename: &str) -> Result<PathBuf, String> {
    let inputs_candidate = inputs_dir().join(filename);
    if inputs_candidate.exists() {
        return Ok(inputs_candidate);
    }

    let sources_candidate = sources_dir().join(filename);
    if sources_candidate.exists() {
        return Ok(sources_candidate);
    }

    Err(format!(
        "File '{}' not found in data/inputs/ or data/sources/",
        filename
    ))
}

/// Ensure data subdirectories exist.
pub fn ensure_data_dirs() -> anyhow::Result<()> {
    for dir in &[data_dir(), sources_dir(), inputs_dir(), exports_dir(), archive_dir()] {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}