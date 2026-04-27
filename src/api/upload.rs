//! Upload API handlers.

use axum::{extract::Multipart, Json};
use serde_json::json;
use crate::paths;

pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let inputs = paths::inputs_dir();
    let mut uploaded = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::api::error::ApiError::BadRequest(format!("Multipart error: {}", e))
    })? {
        let filename = field.file_name()
            .ok_or_else(|| crate::api::error::ApiError::BadRequest("No filename".to_string()))?
            .to_string();
        
        let data = field.bytes().await.map_err(|e| {
            crate::api::error::ApiError::BadRequest(format!("Read error: {}", e))
        })?;

        // Determine target filename
        let target_name = if filename.contains("ECONOMICS") {
            "Listino_CVM_ECONOMICS_UPLOADED.xlsx"
        } else if filename.contains("FASCE") {
            "Listino_CVM_FASCE_UPLOADED.xlsx"
        } else if filename.ends_with(".xlsb") {
            "output_TI_CVM_UPLOADED.xlsb"
        } else if filename.contains("output") || filename.contains("TI_CVM") {
            // Also accept .xlsx version of output
            "output_TI_CVM_UPLOADED.xlsx"
        } else {
            &filename.clone()
        };

        let dest = inputs.join(target_name);
        tokio::fs::write(&dest, &data).await.map_err(|e| {
            crate::api::error::ApiError::Internal(format!("Write error: {}", e))
        })?;

        uploaded.push(json!({
            "original": filename,
            "saved_as": target_name,
            "size": data.len(),
        }));
    }

    Ok(Json(json!({
        "status": "ok",
        "files": uploaded,
        "message": "Files saved. Restart or reload engine to apply changes.",
    })))
}