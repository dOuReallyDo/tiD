//! Core types shared across the engine.
//!
//! Direct Rust port of tiC's dataclass/Dict structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cashflow shape — mirrors tiC's CashflowShape dataclass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashflowShape {
    pub fascia: f64,
    pub anticipo: f64,
    pub importo_smart: f64,
    pub rata_hs: f64,
    pub rata_smart: f64,
    pub ultima_rata: f64,
    pub sconto_mese: f64,
    pub duration: u32,
}

impl Default for CashflowShape {
    fn default() -> Self {
        Self {
            fascia: 0.0,
            anticipo: 0.0,
            importo_smart: 0.0,
            rata_hs: 0.0,
            rata_smart: 0.0,
            ultima_rata: 0.0,
            sconto_mese: 0.0,
            duration: 30,
        }
    }
}

/// KPI result for a single product+mode combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiResult {
    pub npv: f64,
    pub npv_installment: f64,
    pub npv_incremental: f64,
    pub bad_debt: f64,
    pub financing_cost: f64,
    pub pb_pl: i32,
    pub pb_cash: i32,
    pub status: String,
    pub status_reason: String,
    pub target_pb: i32,
    pub monthly_net: f64,
    pub cashflow: CashflowShape,
    pub net_arpu: f64,
    pub commission: f64,
    pub act_fee: f64,
    pub dealer_credit_note: f64,
}

/// Excel baseline KPI values (for compliance check).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExcelKpis {
    pub npv: Option<f64>,
    pub pb_pl: Option<f64>,
    pub pb_cash: Option<f64>,
}

/// Per-mode state for a product — mirrors tiC's mode_state dict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeState {
    pub status: String,
    pub fascia: f64,
    pub anticipo: f64,
    pub importo_smart: f64,
    pub sconto_tariffa: f64,
    pub rata_hs: f64,
    pub rata_smart: f64,
    pub ultima_rata: f64,
    pub sconto: f64,
    pub duration: u32,
    pub kpis: Option<KpiResult>,
    pub excel_kpis: ExcelKpis,
    #[serde(skip)]
    pub rule: Option<FasceRule>,
}

impl Default for ModeState {
    fn default() -> Self {
        Self {
            status: String::new(),
            fascia: 0.0,
            anticipo: 0.0,
            importo_smart: 0.0,
            sconto_tariffa: 0.0,
            rata_hs: 0.0,
            rata_smart: 0.0,
            ultima_rata: 0.0,
            sconto: 0.0,
            duration: 30,
            kpis: None,
            excel_kpis: ExcelKpis::default(),
            rule: None,
        }
    }
}

/// Product record — mirrors tiC's product dict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub tp: f64,
    pub full_price: f64,
    pub cluster: String,
    pub profile: String,
    pub duration: u32,
    pub modes: HashMap<String, ModeState>,
}

/// Fasce rule — mirrors tiC's rule dict from RULE_COLUMN_MAP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FasceRule {
    pub mode: String,
    pub fascia: f64,
    pub status: String,
    pub anticipo: f64,
    pub rata_hs: f64,
    pub rata_smart: f64,
    pub sconto: f64,
    pub mdp: f64,
    pub chiave: String,
}

/// Assumption entry from the ASSUMPTIONS sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub cell: String,
    pub value: f64,
    pub label: String,
}

/// Churn curves — action and no_action arrays (42 months).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnCurves {
    pub action: Vec<f64>,
    pub no_action: Vec<f64>,
}

/// Compliance check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub score_globale: f64,
    pub checked: usize,
    pub passed: usize,
    pub per_mode: HashMap<String, ModeScore>,
    pub per_kpi: HashMap<String, KpiScore>,
    pub mismatches: Vec<Mismatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeScore {
    pub score: f64,
    pub checked: usize,
    pub passed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiScore {
    pub score: f64,
    pub checked: usize,
    pub passed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mismatch {
    pub product_id: String,
    pub mode: String,
    pub kpi: String,
    pub expected: f64,
    pub actual: f64,
    pub delta: f64,
}

/// Mode identifier constants.
pub const MODES: &[&str] = &[
    "VAR_CC",
    "VAR_RID",
    "FIN_COMPASS",
    "FIN_FINDO",
    "RELOAD_CC",
    "RELOAD_COMP",
    "RELOAD_FINDO",
];

/// Column map per mode — 0-indexed positions in LISTINO_CVM sheet.
/// Mirrors tiC's MODE_COLUMN_MAP (updated with rata_hs/rata_smart/ultima_rata).
pub fn mode_column_map() -> HashMap<&'static str, HashMap<&'static str, usize>> {
    let mut map: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    
    let entries: [(&str, [(usize, &[&str]); 8]); 7] = [
        ("VAR_CC",       [(26, &["status"]), (27, &["fascia"]), (28, &["anticipo"]), (29, &["importo_smart"]), (30, &["sconto_tariffa"]), (31, &["rata_hs"]), (32, &["rata_smart"]), (33, &["ultima_rata"])]),
        ("VAR_RID",      [(52, &["status"]), (53, &["fascia"]), (54, &["anticipo"]), (55, &["importo_smart"]), (56, &["sconto_tariffa"]), (57, &["rata_hs"]), (58, &["rata_smart"]), (59, &["ultima_rata"])]),
        ("FIN_COMPASS",  [(78, &["status"]), (79, &["fascia"]), (80, &["anticipo"]), (81, &["importo_smart"]), (82, &["sconto_tariffa"]), (83, &["rata_hs"]), (84, &["rata_smart"]), (85, &["ultima_rata"])]),
        ("FIN_FINDO",   [(110, &["status"]), (111, &["fascia"]), (112, &["anticipo"]), (113, &["importo_smart"]), (114, &["sconto_tariffa"]), (115, &["rata_hs"]), (116, &["rata_smart"]), (117, &["ultima_rata"])]),
        ("RELOAD_CC",   [(142, &["status"]), (143, &["fascia"]), (144, &["anticipo"]), (145, &["importo_smart"]), (146, &["sconto_tariffa"]), (147, &["rata_hs"]), (148, &["rata_smart"]), (149, &["ultima_rata"])]),
        ("RELOAD_COMP", [(171, &["status"]), (172, &["fascia"]), (173, &["anticipo"]), (174, &["importo_smart"]), (175, &["sconto_tariffa"]), (176, &["rata_hs"]), (177, &["rata_smart"]), (178, &["ultima_rata"])]),
        ("RELOAD_FINDO",[(200, &["status"]), (201, &["fascia"]), (202, &["anticipo"]), (203, &["importo_smart"]), (204, &["sconto_tariffa"]), (205, &["rata_hs"]), (206, &["rata_smart"]), (207, &["ultima_rata"])]),
    ];

    for (mode, cols) in entries {
        let mut inner: HashMap<&str, usize> = HashMap::new();
        for (pos, keys) in cols {
            for &key in keys {
                inner.insert(key, pos);
            }
        }
        map.insert(mode, inner);
    }
    map
}

/// KPI column positions (0-indexed) in LISTINO_CVM for each mode.
pub fn kpi_column_map() -> HashMap<&'static str, HashMap<&'static str, usize>> {
    let mut map: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    
    let entries: [(&str, [(&str, usize); 3]); 7] = [
        ("VAR_CC",       [("pb_pl", 43), ("npv", 44), ("pb_cash", 49)]),
        ("VAR_RID",      [("pb_pl", 69), ("npv", 70), ("pb_cash", 75)]),
        ("FIN_COMPASS",  [("pb_pl", 95), ("npv", 96), ("pb_cash", 101)]),
        ("FIN_FINDO",    [("pb_pl", 127), ("npv", 128), ("pb_cash", 133)]),
        ("RELOAD_CC",    [("pb_pl", 159), ("npv", 160), ("pb_cash", 165)]),
        ("RELOAD_COMP",  [("pb_pl", 188), ("npv", 189), ("pb_cash", 194)]),
        ("RELOAD_FINDO", [("pb_pl", 217), ("npv", 218), ("pb_cash", 223)]),
    ];

    for (mode, cols) in entries {
        let mut inner: HashMap<&str, usize> = HashMap::new();
        for (key, pos) in cols {
            inner.insert(key, pos);
        }
        map.insert(mode, inner);
    }
    map
}