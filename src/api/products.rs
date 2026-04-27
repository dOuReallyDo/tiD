//! Products API handlers.

use axum::{extract::{Path, State}, Json};
use serde_json::json;
use crate::api::error::ApiError;
use crate::engine::pricing::SharedEngine;
use crate::engine::versioning::VersionManager;

pub async fn list_products(
    State(engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    let products: Vec<serde_json::Value> = engine.products.iter().map(|p| {
        json!({
            "id": p.id,
            "name": p.name,
            "tp": p.tp,
            "cluster": p.cluster,
            "modes": p.modes.iter().map(|(m, ms)| {
                let kpis = ms.kpis.as_ref();
                json!({
                    m: {
                        "fascia": ms.fascia,
                        "anticipo": ms.anticipo,
                        "status": ms.status,
                        "kpis": kpis.map(|k| json!({
                            "npv": k.npv,
                            "npv_installment": k.npv_installment,
                            "npv_incremental": k.npv_incremental,
                            "bad_debt": k.bad_debt,
                            "financing_cost": k.financing_cost,
                            "pb_pl": k.pb_pl,
                            "pb_cash": k.pb_cash,
                            "status": k.status,
                            "status_reason": k.status_reason,
                            "target_pb": k.target_pb,
                            "monthly_net": k.monthly_net,
                            "net_arpu": k.net_arpu,
                            "commission": k.commission,
                            "act_fee": k.act_fee,
                            "dealer_credit_note": k.dealer_credit_note,
                        })).unwrap_or(json!(null)),
                    }
                })
            }).collect::<Vec<_>>(),
        })
    }).collect();

    Ok(Json(json!({ "products": products, "count": products.len() })))
}

pub async fn get_product(
    State(engine): State<SharedEngine>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    let product = engine.get_product(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product not found: {}", id)))?;

    Ok(Json(json!({
        "id": product.id,
        "name": product.name,
        "tp": product.tp,
        "full_price": product.full_price,
        "cluster": product.cluster,
        "profile": product.profile,
        "duration": product.duration,
        "modes": product.modes,
    })))
}

#[derive(serde::Deserialize)]
pub struct EditRequest {
    pub mode: String,
    pub field: String,
    pub value: f64,
}

pub async fn edit_product(
    State(engine): State<SharedEngine>,
    Path(id): Path<String>,
    Json(body): Json<EditRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut engine = engine.write().await;
    engine.edit_product(&id, &body.mode, &body.field, body.value)?;

    let product = engine.get_product(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product not found: {}", id)))?;

    Ok(Json(json!({
        "id": product.id,
        "mode": body.mode,
        "field": body.field,
        "value": body.value,
        "status": "updated",
    })))
}

#[derive(serde::Deserialize)]
pub struct ApproveRequest {
    pub label: Option<String>,
}

pub async fn approve_product(
    State(engine): State<SharedEngine>,
    Path(id): Path<String>,
    Json(body): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let engine = engine.read().await;
    let _product = engine.get_product(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product not found: {}", id)))?;

    // Create version snapshot
    let label = body.label.unwrap_or_else(|| format!("approve-{}", id));
    let version = VersionManager::snapshot(&engine.products, &engine.assumptions, &label)?;

    // Mark as approved
    VersionManager::approve_version(&version.id)?;

    Ok(Json(json!({
        "id": id,
        "status": "approved",
        "version_id": version.id,
        "version_label": version.label,
        "timestamp": version.timestamp,
    })))
}

#[derive(serde::Deserialize)]
pub struct BatchEditRequest {
    pub product_ids: Vec<String>,
    pub mode: String,
    pub field: String,
    pub value: f64,
}

pub async fn batch_edit(
    State(engine): State<SharedEngine>,
    Json(body): Json<BatchEditRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut engine = engine.write().await;
    let mut updated = Vec::new();

    for id in &body.product_ids {
        if engine.edit_product(id, &body.mode, &body.field, body.value).is_ok() {
            updated.push(id.clone());
        }
    }

    Ok(Json(json!({
        "updated": updated,
        "count": updated.len(),
    })))
}

pub async fn list_versions(
    State(_engine): State<SharedEngine>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let versions = VersionManager::list_versions()?;

    Ok(Json(json!({
        "versions": versions,
        "count": versions.len(),
    })))
}