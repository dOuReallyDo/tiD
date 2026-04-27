//! Compliance API handler.

use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use crate::api::error::ApiError;
use crate::engine::pricing::SharedEngine;

#[derive(Deserialize)]
pub struct ComplianceRequest {
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_tolerance() -> f64 {
    0.01
}

pub async fn run_compliance(
    State(engine): State<SharedEngine>,
    Json(body): Json<ComplianceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    let result = engine.run_compliance(body.tolerance);
    
    Ok(Json(json!({
        "score_globale": result.score_globale,
        "checked": result.checked,
        "passed": result.passed,
        "pass": result.score_globale >= 80.0,
        "per_mode": result.per_mode,
        "per_kpi": result.per_kpi,
        "mismatches": result.mismatches,
    })))
}