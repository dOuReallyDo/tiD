//! Export API handlers.

use axum::{extract::State, Json};
use serde_json::json;
use crate::engine::pricing::SharedEngine;
use crate::export::writer::WorkbookWriter;
use crate::export::compare::WorkbookCompare;
use crate::export::package;
use crate::paths;

pub async fn export_economics(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let engine = engine.read().await;
    let exports = paths::exports_dir();
    let output_path = exports.join("economics_export.xlsx");

    WorkbookWriter::write_economics(&engine.products, &engine.assumptions, &output_path)?;

    Ok(Json(json!({
        "status": "ok",
        "file": output_path.to_string_lossy(),
        "products": engine.products.len(),
    })))
}

pub async fn export_fasce(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let engine = engine.read().await;
    let exports = paths::exports_dir();
    let output_path = exports.join("fasce_export.xlsx");

    WorkbookWriter::write_fasce(&engine.products, &output_path)?;

    Ok(Json(json!({
        "status": "ok",
        "file": output_path.to_string_lossy(),
    })))
}

pub async fn export_fasce_request(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let engine = engine.read().await;
    let exports = paths::exports_dir();
    let output_path = exports.join("fasce_request_export.xlsx");

    WorkbookWriter::write_fasce(&engine.products, &output_path)?;

    Ok(Json(json!({
        "status": "ok",
        "file": output_path.to_string_lossy(),
        "type": "fasce_request",
    })))
}

pub async fn export_fasce_config(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let engine = engine.read().await;
    let exports = paths::exports_dir();
    let output_path = exports.join("fasce_config_export.xlsx");

    WorkbookWriter::write_fasce(&engine.products, &output_path)?;

    Ok(Json(json!({
        "status": "ok",
        "file": output_path.to_string_lossy(),
        "type": "fasce_config",
    })))
}

pub async fn export_full_package(
    State(_engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let zip_path = package::build_full_package()?;

    Ok(Json(json!({
        "status": "ok",
        "file": zip_path.to_string_lossy(),
        "type": "full_package",
    })))
}

pub async fn compare_workbooks(
    State(_engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    // Find source and exported files
    let inputs = paths::inputs_dir();
    let sources = paths::sources_dir();
    let exports = paths::exports_dir();

    // Look for source ECONOMICS.xlsx — prefer uploaded, fall back to factory
    let uploaded = inputs.join("ECONOMICS_UPLOADED.xlsx");
    let factory = sources.join("ECONOMICS.xlsx");
    let source_path = if uploaded.exists() {
        uploaded
    } else if factory.exists() {
        factory
    } else {
        return Err(crate::api::error::ApiError::NotFound(
            "Source ECONOMICS.xlsx not found".to_string()
        ));
    };

    let exported_path = exports.join("economics_export.xlsx");
    if !exported_path.exists() {
        return Err(crate::api::error::ApiError::NotFound(
            "Exported economics_export.xlsx not found — run export first".to_string()
        ));
    }

    let summary = WorkbookCompare::compare(&source_path, &exported_path)?;

    Ok(Json(json!({
        "status": "ok",
        "summary": {
            "total_cells": summary.total_cells,
            "matching_cells": summary.matching_cells,
            "diff_cells": summary.diff_cells,
            "sheets_compared": summary.sheets_compared,
        },
        "diffs": summary.diffs,
    })))
}