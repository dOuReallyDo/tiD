//! Version Manager — Snapshot, list, and approve product state versions.

use crate::engine::types::*;
use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A saved version snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub label: String,
    pub timestamp: String,
    pub approved: bool,
    pub products_json: String,
    pub assumptions_json: String,
}

/// Lightweight version info for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub label: String,
    pub timestamp: String,
    pub approved: bool,
}

pub struct VersionManager;

impl VersionManager {
    /// Save a snapshot of current engine state to archive.
    pub fn snapshot(
        products: &[Product],
        assumptions: &HashMap<String, Assumption>,
        label: &str,
    ) -> anyhow::Result<Version> {
        let archive = paths::archive_dir();
        std::fs::create_dir_all(&archive)?;

        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let id = format!("v-{}-{}", &timestamp[..15], uuid::Uuid::new_v4().as_simple());

        let products_json = serde_json::to_string(products)?;
        let assumptions_json = serde_json::to_string(assumptions)?;

        let version = Version {
            id: id.clone(),
            label: label.to_string(),
            timestamp: timestamp.clone(),
            approved: false,
            products_json,
            assumptions_json,
        };

        // Save to file
        let version_path = archive.join(format!("{}.json", id));
        let json = serde_json::to_string_pretty(&version)?;
        std::fs::write(version_path, json)?;

        Ok(version)
    }

    /// List all saved versions (sorted by timestamp, newest first).
    pub fn list_versions() -> anyhow::Result<Vec<VersionInfo>> {
        let archive = paths::archive_dir();
        if !archive.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(archive)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(version) = serde_json::from_str::<Version>(&content) {
                        versions.push(VersionInfo {
                            id: version.id,
                            label: version.label,
                            timestamp: version.timestamp,
                            approved: version.approved,
                        });
                    }
                }
            }
        }

        // Sort newest first
        versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(versions)
    }

    /// Load a specific version by ID.
    pub fn get_version(id: &str) -> anyhow::Result<Option<Version>> {
        let archive = paths::archive_dir();
        let version_path = archive.join(format!("{}.json", id));
        if !version_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(version_path)?;
        let version = serde_json::from_str(&content)?;
        Ok(Some(version))
    }

    /// Mark a version as approved.
    pub fn approve_version(id: &str) -> anyhow::Result<()> {
        let archive = paths::archive_dir();
        let version_path = archive.join(format!("{}.json", id));
        if !version_path.exists() {
            return Err(anyhow::anyhow!("Version not found: {}", id));
        }

        let content = std::fs::read_to_string(&version_path)?;
        let mut version: Version = serde_json::from_str(&content)?;
        version.approved = true;

        let json = serde_json::to_string_pretty(&version)?;
        std::fs::write(version_path, json)?;
        Ok(())
    }
}