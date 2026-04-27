//! Export API handlers.

use axum::{extract::State, Json};
use serde_json::json;
use crate::engine::pricing::SharedEngine;
use crate::export::writer::WorkbookWriter;
use crate::paths;

pub async fn export_economics(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let engine = engine.read().await;
    let exports = paths::exports_dir();
    let output_path = exports.join("economics_export.xlsx");
    
    WorkbookWriter::write_economics(&engine.products, &output_path)?;
    
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
    State(_engine): State<SharedEngine>,
) -> Json<serde_json::Value> {
    Json(json!({ "status": "not_implemented", "message": "fasce_request export coming in v0.4" }))
}

pub async fn export_fasce_config(
    State(_engine): State<SharedEngine>,
) -> Json<serde_json::Value> {
    Json(json!({ "status": "not_implemented", "message": "fasce_config export coming in v0.4" }))
}

pub async fn export_full_package(
    State(_engine): State<SharedEngine>,
) -> Json<serde_json::Value> {
    Json(json!({ "status": "not_implemented", "message": "full-package export coming in v0.4" }))
}