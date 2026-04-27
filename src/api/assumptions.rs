//! Assumptions API handlers.

use axum::{extract::State, Json};
use serde_json::json;
use crate::api::error::ApiError;
use crate::engine::pricing::SharedEngine;

pub async fn get_assumptions(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    let assumptions: serde_json::Map<String, serde_json::Value> = engine.assumptions
        .iter()
        .map(|(k, v)| (k.clone(), json!({ "cell": v.cell, "value": v.value, "label": v.label })))
        .collect();
    
    Ok(Json(json!({ "assumptions": assumptions })))
}

#[derive(serde::Deserialize)]
pub struct UpdateAssumptionsRequest {
    pub assumptions: std::collections::HashMap<String, f64>,
}

pub async fn update_assumptions(
    State(engine): State<SharedEngine>,
    Json(body): Json<UpdateAssumptionsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut engine = engine.write().await;
    let mut updated = Vec::new();

    for (key, value) in body.assumptions {
        if let Some(a) = engine.assumptions.get_mut(&key) {
            a.value = value;
            updated.push(key);
        }
    }

    // Recalculate all KPIs with new assumptions
    let products_count = engine.products.len();
    for i in 0..engine.products.len() {
        for mode in crate::engine::types::MODES.iter().map(|m| m.to_string()) {
            if let Some(ms) = engine.products[i].modes.get(&mode) {
                let kpis = engine.economics.calculate_mode_kpis(&engine.products[i], &mode, ms);
                engine.products[i].modes.get_mut(&mode).unwrap().kpis = Some(kpis);
            }
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "updated": updated,
        "products_recalculated": products_count,
    })))
}