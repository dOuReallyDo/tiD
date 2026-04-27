//! Unit tests for the Economics Engine.
//!
//! Validates that Rust KPI calculations match tiC's Python implementation.

use tid::engine::economics::*;
use tid::engine::types::*;
use std::collections::HashMap;

fn make_assumptions() -> HashMap<String, Assumption> {
    vec![
        ("BAD_DEBT_RID", "B4", 0.0936),
        ("BAD_DEBT_CC", "B5", 0.081),
        ("RATE_COMPASS", "B6", 0.083),
        ("RATE_FINDO", "B7", 0.0805),
        ("RATE_COMPASS_CUSTOMER", "D6", 0.035),
        ("RATE_FINDO_CUSTOMER", "D7", 0.035),
        ("NA_A", "B10", 22.0),
        ("NA_B", "B11", 19.0),
        ("NA_C", "B12", 16.0),
        ("NA_D", "B13", 13.0),
        ("NA_E", "B14", 10.0),
        ("NA_CB", "B15", 15.0),
        ("NA_NT", "B16", 9.0),
        ("COMM_VAR", "B25", 20.0),
        ("COMM_FIN", "B26", 30.0),
        ("COMM_RPLUS", "B27", 15.0),
        ("ACT_FEE", "B28", 6.99),
        ("TARGET_PB_VAR", "B30", 12.0),
        ("TARGET_PB_FIN", "B31", 16.0),
    ]
    .into_iter()
    .map(|(key, cell, value)| (key.to_string(), Assumption { cell: cell.to_string(), value, label: key.to_string() }))
    .collect()
}

fn make_product(id: &str, tp: f64, cluster: &str, full_price: f64) -> Product {
    Product {
        id: id.to_string(),
        name: format!("Product {}", id),
        tp,
        full_price,
        cluster: cluster.to_string(),
        profile: String::new(),
        duration: 30,
        modes: HashMap::new(),
    }
}

fn make_mode_state(fascia: f64, anticipo: f64, rata_hs: f64) -> ModeState {
    ModeState {
        fascia,
        anticipo,
        importo_smart: 5.0,
        rata_hs,
        rata_smart: 5.0,
        ultima_rata: 0.0,
        sconto: 0.0,
        sconto_tariffa: 0.0,
        duration: 30,
        kpis: None,
        excel_kpis: ExcelKpis::default(),
        rule: None,
        status: "ATTIVO".to_string(),
    }
}

// ── Cashflow Structure Tests ─────────────────────────────────────

#[test]
fn test_cashflow_30m() {
    let engine = EconomicsEngine::with_empty();
    let ms = make_mode_state(600.0, 100.0, 20.0);
    let shape = engine.build_cashflow_shape(&ms);
    
    assert_eq!(shape.duration, 30);
    assert_eq!(shape.fascia, 600.0);
    assert_eq!(shape.anticipo, 100.0);
    assert_eq!(shape.rata_hs, 20.0);
    
    let cfs = engine.build_cashflows(&shape, 500.0, 20.0);
    
    // 30m: CF0 + 24*monthly + 5*rata_smart + ultima_rata = 31 CFs
    assert_eq!(cfs.len(), 31);
    // CF0 = -TP + anticipo = -500 + 100 = -400
    assert!((cfs[0] - (-400.0)).abs() < 0.001);
}

#[test]
fn test_cashflow_24m() {
    let engine = EconomicsEngine::with_empty();
    let mut ms = make_mode_state(480.0, 80.0, 16.67);
    ms.duration = 24;
    let shape = engine.build_cashflow_shape(&ms);
    
    assert_eq!(shape.duration, 24);
    
    let cfs = engine.build_cashflows(&shape, 400.0, 16.67);
    // 24m: CF0 + 23*monthly + ultima_rata = 25 CFs
    assert_eq!(cfs.len(), 25);
    // CF0 = -TP + anticipo = -400 + 80 = -320
    assert!((cfs[0] - (-320.0)).abs() < 0.01);
}

#[test]
fn test_cashflow_36m() {
    let engine = EconomicsEngine::with_empty();
    let mut ms = make_mode_state(720.0, 120.0, 16.67);
    ms.duration = 36;
    let shape = engine.build_cashflow_shape(&ms);
    
    assert_eq!(shape.duration, 36);
    
    let cfs = engine.build_cashflows(&shape, 600.0, 16.67);
    // 36m: CF0 + 35*monthly + ultima_rata = 37 CFs
    assert_eq!(cfs.len(), 37);
}

// ── NPV Tests ────────────────────────────────────────────────────

#[test]
fn test_npv_zero_rate() {
    // With 0% discount rate, NPV = sum of cashflows
    let cfs = vec![-100.0, 10.0, 10.0, 10.0, 10.0, 80.0];
    let result = npv(&cfs, 0.0);
    assert!((result - 20.0).abs() < 0.001);
}

#[test]
fn test_npv_positive_rate() {
    let cfs = vec![-100.0, 30.0, 30.0, 30.0, 30.0];
    let result = npv(&cfs, 0.01);
    // NPV should be positive but less than sum of inflows
    assert!(result > 0.0);
    assert!(result < 120.0);
}

// ── Payback Tests ────────────────────────────────────────────────

#[test]
fn test_payback_simple() {
    let cfs = vec![-100.0, 30.0, 30.0, 30.0, 30.0];
    let pb = payback(&cfs);
    // Cumulative: -100, -70, -40, -10, +20 → month 4
    assert_eq!(pb, 4);
}

#[test]
fn test_payback_never() {
    let cfs = vec![-100.0, 10.0, 10.0, 10.0];
    let pb = payback(&cfs);
    assert_eq!(pb, -1);
}

#[test]
fn test_payback_immediate() {
    let cfs = vec![50.0, 10.0, 10.0];
    let pb = payback(&cfs);
    assert_eq!(pb, 0);
}

// ── Bad Debt (Montante-based) ────────────────────────────────────

#[test]
fn test_bad_debt_var_rid() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    // VAR_RID: BAD_DEBT_RID = 9.36%, montante = fascia - anticipo
    let bd = engine.bad_debt_adjustment("VAR_RID", 600.0, 100.0);
    let expected = (600.0 - 100.0) * 0.0936; // 500 * 0.0936 = 46.8
    assert!((bd - expected).abs() < 0.01);
}

#[test]
fn test_bad_debt_var_cc() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    // VAR_CC: BAD_DEBT_CC = 8.1%
    let bd = engine.bad_debt_adjustment("VAR_CC", 600.0, 100.0);
    let expected = (600.0 - 100.0) * 0.081; // 500 * 0.081 = 40.5
    assert!((bd - 40.5).abs() < 0.01);
}

#[test]
fn test_bad_debt_fin_zero() {
    // FIN modes: bad debt is 0 (captured by financing cost)
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let bd = engine.bad_debt_adjustment("FIN_COMPASS", 600.0, 100.0);
    assert!((bd - 0.0).abs() < 0.001);
}

// ── Financing Cost (Declining Balance) ───────────────────────────

#[test]
fn test_financing_cost_compass() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    // FIN_COMPASS: spread = 8.3% - 3.5% = 4.8%
    let fc = engine.financing_cost("FIN_COMPASS", 600.0, 100.0, 30, 16.67);
    // Should be positive (declining balance NPV of spread)
    assert!(fc > 0.0);
    // Should be less than simple: fascia * spread * duration/12
    let simple = 500.0 * 0.048 * 30.0 / 12.0;
    assert!(fc < simple);
}

#[test]
fn test_financing_cost_var_zero() {
    // VAR modes: no financing cost
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let fc = engine.financing_cost("VAR_CC", 600.0, 100.0, 30, 20.0);
    assert!((fc - 0.0).abs() < 0.001);
}

// ── Trunc function ───────────────────────────────────────────────

#[test]
fn test_trunc() {
    assert!((trunc(123.456789, 6) - 123.456789).abs() < 0.0001);
    assert!((trunc(123.456789, 2) - 123.45).abs() < 0.001);
    assert!((trunc(0.0, 6)).abs() < 0.0001);
}

// ── Net ARPU (cluster-based) ─────────────────────────────────────

#[test]
fn test_net_arpu_cluster() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let product = make_product("P1", 500.0, "A", 0.0);
    let arpu = engine.net_arpu(&product);
    // Cluster A → NA_A = 22.0
    assert!((arpu - 22.0).abs() < 0.01);
}

#[test]
fn test_net_arpu_cluster_cb() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let product = make_product("P2", 500.0, "CB", 0.0);
    let arpu = engine.net_arpu(&product);
    // Cluster CB → NA_CB = 15.0
    assert!((arpu - 15.0).abs() < 0.01);
}

// ── Full KPI calculation (integration) ───────────────────────────

#[test]
fn test_calculate_mode_kpis_var_cc() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let product = make_product("TEST1", 500.0, "A", 0.0);
    let mode_state = make_mode_state(600.0, 100.0, 20.0);
    
    let result = engine.calculate_mode_kpis(&product, "VAR_CC", &mode_state);
    
    // KPIs should be calculated
    assert!(!result.status.is_empty());
    // NPV should be a finite number
    assert!(result.npv.is_finite());
    // Payback should be a valid month or -1
    assert!(result.pb_pl >= -1);
    assert!(result.pb_cash >= -1);
}

#[test]
fn test_calculate_mode_kpis_fin_compass() {
    let assumptions = make_assumptions();
    let engine = EconomicsEngine::new(assumptions, None);
    
    let product = make_product("TEST2", 800.0, "B", 0.0);
    let mut ms = make_mode_state(900.0, 200.0, 23.33);
    ms.sconto = 0.0;
    
    let result = engine.calculate_mode_kpis(&product, "FIN_COMPASS", &ms);
    
    // FIN mode should have financing_cost > 0
    assert!(result.financing_cost > 0.0);
    // FIN mode target_pb should be 16
    assert_eq!(result.target_pb, 16);
}