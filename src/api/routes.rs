//! API routes — Axum router definition.

use axum::{routing::{get, post}, Router, Json};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use serde_json::json;

use crate::engine::pricing::SharedEngine;
use crate::api::products;
use crate::api::export;
use crate::api::upload;
use crate::api::assumptions;
use crate::api::churn;
use crate::api::compliance;

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub fn create_router(engine: SharedEngine) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/products", get(products::list_products))
        .route("/api/product/{id}", get(products::get_product))
        .route("/api/product/{id}/edit", post(products::edit_product))
        .route("/api/product/{id}/approve", post(products::approve_product))
        .route("/api/export/economics", post(export::export_economics))
        .route("/api/export/fasce", post(export::export_fasce))
        .route("/api/export/fasce_request", post(export::export_fasce_request))
        .route("/api/export/fasce_config", post(export::export_fasce_config))
        .route("/api/export/full-package", post(export::export_full_package))
        .route("/api/upload", post(upload::upload_file))
        .route("/api/assumptions", get(assumptions::get_assumptions).post(assumptions::update_assumptions))
        .route("/api/churn", get(churn::get_churn).post(churn::update_churn))
        .route("/api/compliance", post(compliance::run_compliance))
        .route("/api/batch-edit", post(products::batch_edit))
        .route("/api/versions", get(products::list_versions))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(engine)
}