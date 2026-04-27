//! Churn curves API handlers.

use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use crate::api::error::ApiError;
use crate::engine::pricing::SharedEngine;

pub async fn get_churn(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    match &engine.churn_curves {
        Some(curves) => Ok(Json(json!({
            "action": curves.action,
            "no_action": curves.no_action,
            "length": curves.action.len(),
        }))),
        None => Ok(Json(json!({
            "action": [],
            "no_action": [],
            "length": 0,
            "message": "No churn curves loaded",
        }))),
    }
}

#[derive(Deserialize)]
pub struct UpdateChurnRequest {
    pub action: Vec<f64>,
    pub no_action: Vec<f64>,
}

pub async fn update_churn(
    State(engine): State<SharedEngine>,
    Json(body): Json<UpdateChurnRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut engine = engine.write().await;
    
    // Validate length
    if body.action.len() != 42 || body.no_action.len() != 42 {
        return Err(ApiError::BadRequest(
            "Churn curves must have exactly 42 values".to_string()
        ));
    }

    let curves = crate::engine::types::ChurnCurves {
        action: body.action,
        no_action: body.no_action,
    };
    
    engine.economics.update_churn_curves(Some(curves.clone()));
    engine.churn_curves = Some(curves);

    // Recalculate all KPIs
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
        "message": "Churn curves updated, all KPIs recalculated",
    })))
}