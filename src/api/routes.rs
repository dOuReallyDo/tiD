//! API routes — Axum router definition.

use axum::{routing::{get, post}, Router, Json};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::services::{ServeDir, ServeFile};
use serde_json::json;

use crate::engine::pricing::SharedEngine;
use crate::api::products;
use crate::api::export;
use crate::api::upload;
use crate::api::assumptions;
use crate::api::churn;
use crate::api::compliance;
use crate::paths;

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub fn create_router(engine: SharedEngine) -> Router {
    let frontend_dir = paths::frontend_dir();
    
    let api_routes = Router::new()
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
        .route("/api/export/compare", post(export::compare_workbooks))
        .route("/api/upload", post(upload::upload_file))
        .route("/api/assumptions", get(assumptions::get_assumptions).post(assumptions::update_assumptions))
        .route("/api/churn", get(churn::get_churn).post(churn::update_churn))
        .route("/api/compliance", post(compliance::run_compliance))
        .route("/api/batch-edit", post(products::batch_edit))
        .route("/api/versions", get(products::list_versions))
        .with_state(engine);

    // SPA fallback: serve index.html when a static file isn't found
    // (required for Vue Router history mode — all non-API, non-file routes
    //  must return index.html so the client-side router can handle them)
    let index_html = frontend_dir.join("index.html");

    Router::new()
        .merge(api_routes)
        // Serve static frontend files with SPA fallback
        .fallback_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(index_html)))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
}