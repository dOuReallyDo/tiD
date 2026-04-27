//! Pricing Engine — Orchestration singleton.
//!
//! Manages product state, versioning, and coordinates
//! EconomicsEngine calculations.

use crate::engine::types::*;
use crate::engine::economics::EconomicsEngine;
use crate::engine::data::DataManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PricingEngine {
    pub products: Vec<Product>,
    pub assumptions: HashMap<String, Assumption>,
    pub churn_curves: Option<ChurnCurves>,
    pub economics: EconomicsEngine,
}

impl PricingEngine {
    /// Load all data from Excel files and build KPIs.
    pub fn load() -> anyhow::Result<Self> {
        let (mut products, assumptions, churn_curves) = DataManager::load_all()?;
        let economics = EconomicsEngine::new(assumptions.clone(), churn_curves.clone());

        // Calculate KPIs for all products — clone mode keys to avoid borrow conflict
        for product in &mut products {
            let mode_names: Vec<String> = product.modes.keys().cloned().collect();
            for m in &mode_names {
                let ms = product.modes.get(m).cloned();
                if let Some(mode_state) = ms {
                    let kpis = economics.calculate_mode_kpis(product, m, &mode_state);
                    product.modes.get_mut(m).unwrap().kpis = Some(kpis);
                }
            }
        }

        Ok(Self {
            products,
            assumptions,
            churn_curves,
            economics,
        })
    }

    /// Recalculate KPIs after a parameter edit.
    pub fn recalculate_product(&mut self, product_id: &str) -> Option<&Product> {
        let idx = self.products.iter().position(|p| p.id == product_id)?;
        let mode_names: Vec<String> = self.products[idx].modes.keys().cloned().collect();
        for m in &mode_names {
            let ms = self.products[idx].modes.get(m).cloned();
            if let Some(mode_state) = ms {
                let kpis = self.economics.calculate_mode_kpis(&self.products[idx], m, &mode_state);
                self.products[idx].modes.get_mut(m).unwrap().kpis = Some(kpis);
            }
        }
        Some(&self.products[idx])
    }

    /// Run compliance check.
    pub fn run_compliance(&self, tolerance: f64) -> ComplianceResult {
        let _col_map = mode_column_map();
        let mut checked = 0usize;
        let mut passed = 0usize;
        let mut per_mode: HashMap<String, ModeScore> = HashMap::new();
        let mut per_kpi: HashMap<String, KpiScore> = HashMap::new();

        // Initialize per-mode and per-kpi counters
        for mode in MODES {
            per_mode.insert(mode.to_string(), ModeScore { score: 0.0, checked: 0, passed: 0 });
        }
        for kpi in &["npv", "pb_pl", "pb_cash"] {
            per_kpi.insert(kpi.to_string(), KpiScore { score: 0.0, checked: 0, passed: 0 });
        }

        let mut mismatches = Vec::new();

        for product in &self.products {
            for mode in MODES {
                let mode_state = match product.modes.get(*mode) {
                    Some(s) => s,
                    None => continue,
                };
                let kpis = match &mode_state.kpis {
                    Some(k) => k,
                    None => continue,
                };
                let baseline = &mode_state.excel_kpis;

                for (kpi_key, actual, expected_opt) in &[
                    ("npv", kpis.npv, &baseline.npv),
                    ("pb_pl", kpis.pb_pl as f64, &baseline.pb_pl),
                    ("pb_cash", kpis.pb_cash as f64, &baseline.pb_cash),
                ] {
                    let expected = match expected_opt {
                        Some(v) => *v,
                        None => continue,
                    };

                    checked += 1;
                    if let Some(ms) = per_mode.get_mut(*mode) { ms.checked += 1; }
                    if let Some(ks) = per_kpi.get_mut(*kpi_key) { ks.checked += 1; }

                    let tol = (1e-6 * expected.abs()).max(tolerance);
                    if (actual - expected).abs() <= tol {
                        passed += 1;
                        if let Some(ms) = per_mode.get_mut(*mode) { ms.passed += 1; }
                        if let Some(ks) = per_kpi.get_mut(*kpi_key) { ks.passed += 1; }
                    } else {
                        mismatches.push(Mismatch {
                            product_id: product.id.clone(),
                            mode: mode.to_string(),
                            kpi: kpi_key.to_string(),
                            expected: (expected * 10000.0).round() / 10000.0,
                            actual: (actual * 10000.0).round() / 10000.0,
                            delta: ((actual - expected).abs() * 10000.0).round() / 10000.0,
                        });
                    }
                }
            }
        }

        // Compute scores
        let score_globale = if checked > 0 { (passed as f64 / checked as f64 * 100.0 * 100.0).round() / 100.0 } else { 0.0 };

        for ms in per_mode.values_mut() {
            ms.score = if ms.checked > 0 { (ms.passed as f64 / ms.checked as f64 * 100.0 * 100.0).round() / 100.0 } else { 0.0 };
        }
        for ks in per_kpi.values_mut() {
            ks.score = if ks.checked > 0 { (ks.passed as f64 / ks.checked as f64 * 100.0 * 100.0).round() / 100.0 } else { 0.0 };
        }

        ComplianceResult {
            score_globale,
            checked,
            passed,
            per_mode,
            per_kpi,
            mismatches,
        }
    }

    /// Get product by ID.
    pub fn get_product(&self, id: &str) -> Option<&Product> {
        self.products.iter().find(|p| p.id == id)
    }

    /// Edit a product parameter and recalculate.
    pub fn edit_product(
        &mut self,
        product_id: &str,
        mode: &str,
        field: &str,
        value: f64,
    ) -> anyhow::Result<()> {
        let idx = self.products.iter().position(|p| p.id == product_id)
            .ok_or_else(|| anyhow::anyhow!("Product not found: {}", product_id))?;

        // Edit the field
        match field {
            "fascia" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.fascia = value,
            "anticipo" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.anticipo = value,
            "importo_smart" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.importo_smart = value,
            "rata_hs" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.rata_hs = value,
            "rata_smart" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.rata_smart = value,
            "sconto" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.sconto = value,
            "sconto_tariffa" => self.products[idx].modes.get_mut(mode).ok_or_else(|| anyhow::anyhow!("Mode not found: {}", mode))?.sconto_tariffa = value,
            _ => return Err(anyhow::anyhow!("Unknown field: {}", field)),
        }

        // Recalculate — clone the modes list to avoid borrow conflict
        let mode_names: Vec<String> = self.products[idx].modes.keys().cloned().collect();
        for m in &mode_names {
            let ms = self.products[idx].modes.get(m).cloned();
            if let Some(mode_state) = ms {
                let kpis = self.economics.calculate_mode_kpis(&self.products[idx], m, &mode_state);
                self.products[idx].modes.get_mut(m).unwrap().kpis = Some(kpis);
            }
        }

        Ok(())
    }
}

/// Thread-safe wrapper for Axum state.
pub type SharedEngine = Arc<RwLock<PricingEngine>>;