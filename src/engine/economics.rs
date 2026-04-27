//! Economics Engine — Pure financial math.
//!
//! Direct Rust port of tiC's economics_engine.py.
//! All formulas preserved exactly for compliance ≥99.9%.

use crate::engine::types::*;
use crate::engine::churn::ChurnEngine;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────

/// Parse any value to f64, handling Italian number formats.
pub fn as_f64(value: &str, default: f64) -> f64 {
    let text = value.trim();
    if text.is_empty() || text == "nan" || text == "NaN" || text == "None" || text == "(vuoto)" {
        return default;
    }
    let mut cleaned = text.to_string();
    // Handle Italian format: 1.234,56 → 1234.56
    if cleaned.contains(',') && cleaned.contains('.') {
        if cleaned.rfind(',') > cleaned.rfind('.') {
            cleaned = cleaned.replace('.', "").replace(',', ".");
        } else {
            cleaned.retain(|c| c != ',');
        }
    } else if cleaned.contains(',') {
        cleaned = cleaned.replace(',', ".");
    }
    cleaned.parse::<f64>().unwrap_or(default)
}

/// Parse numeric value from calamine CellType, returning default for non-numeric.
pub fn cell_as_f64(cell: &calamine::Data, default: f64) -> f64 {
    match cell {
        calamine::Data::Float(f) => *f,
        calamine::Data::Int(i) => *i as f64,
        calamine::Data::String(s) => as_f64(s, default),
        _ => default,
    }
}

/// Truncate to N decimal places (same as tiC's _trunc).
pub fn trunc(value: f64, decimals: i32) -> f64 {
    let decimals = decimals.max(0) as i32;
    let factor = 10_f64.powi(decimals);
    (value * factor).trunc() / factor
}

// ── Mapping tables (identical to tiC) ──────────────────────────

/// Net ARPU assumption key per cluster.
const PROFILE_ARPU_MAP: &[(&str, &str)] = &[
    ("1", "NA_A"), ("A", "NA_A"),
    ("2", "NA_B"), ("B", "NA_B"),
    ("3", "NA_C"), ("C", "NA_C"),
    ("4", "NA_D"), ("D", "NA_D"),
    ("5", "NA_E"), ("E", "NA_E"),
    ("CB", "NA_CB"),
    ("NT", "NA_NT"),
];

/// Commission assumption key per mode.
const MODE_COMMISSION_MAP: &[(&str, &str)] = &[
    ("VAR_CC", "COMM_VAR"),
    ("VAR_RID", "COMM_VAR"),
    ("FIN_COMPASS", "COMM_FIN"),
    ("FIN_FINDO", "COMM_FIN"),
    ("RELOAD_CC", "COMM_RPLUS"),
    ("RELOAD_COMP", "COMM_RPLUS"),
    ("RELOAD_FINDO", "COMM_RPLUS"),
];

/// Bad debt mode map.
const BAD_DEBT_MODE_MAP: &[(&str, &str)] = &[
    ("VAR_RID", "BAD_DEBT_RID"),
    ("VAR_CC", "BAD_DEBT_CC"),
    ("RELOAD_CC", "BAD_DEBT_CC"),
    ("RELOAD_COMP", "BAD_DEBT_CC"),
    ("RELOAD_FINDO", "BAD_DEBT_CC"),
    // FIN modes: risk captured by financing spread
];

/// Financing rate keys per FIN mode.
const FIN_RATE_MAP: &[(&str, &str, &str)] = &[
    ("FIN_COMPASS", "RATE_COMPASS", "RATE_COMPASS_CUSTOMER"),
    ("FIN_FINDO", "RATE_FINDO", "RATE_FINDO_CUSTOMER"),
];

fn lookup_str<'a>(map: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn lookup_fin(key: &str) -> Option<(&str, &str)> {
    FIN_RATE_MAP.iter().find(|(k, _, _)| *k == key).map(|(_, a, b)| (*a, *b))
}

// ── EconomicsEngine ──────────────────────────────────────────────

pub struct EconomicsEngine {
    assumptions: HashMap<String, Assumption>,
    churn_curves: Option<ChurnCurves>,
}

impl EconomicsEngine {
    pub fn new(
        assumptions: HashMap<String, Assumption>,
        churn_curves: Option<ChurnCurves>,
    ) -> Self {
        Self { assumptions, churn_curves }
    }

    pub fn with_empty() -> Self {
        Self {
            assumptions: HashMap::new(),
            churn_curves: None,
        }
    }

    pub fn update_assumptions(&mut self, assumptions: HashMap<String, Assumption>) {
        self.assumptions = assumptions;
    }

    pub fn update_churn_curves(&mut self, curves: Option<ChurnCurves>) {
        self.churn_curves = curves;
    }

    /// Main KPI calculation — mirrors tiC's _calculate_mode_kpis.
    pub fn calculate_mode_kpis(
        &self,
        product: &Product,
        mode: &str,
        mode_state: &ModeState,
    ) -> KpiResult {
        let tp = trunc(product.tp, 6);
        let shape = self.build_cashflow_shape(mode_state);
        let annual_rate = trunc(self.annual_rate(mode), 6);
        let monthly_rate = trunc((1.0 + annual_rate).powf(1.0 / 12.0) - 1.0, 6);
        let target_pb = self.target_pb(mode) as i32;
        let monthly_net = trunc(shape.rata_hs - shape.sconto_mese, 6);

        // Context metrics
        let net_arpu = self.net_arpu(product);
        let commission = self.commission(mode);
        let act_fee = self.assumption("ACT_FEE", 6.99);
        let street_price = product.full_price;
        let dealer_credit_note = if street_price > 0.0 && shape.fascia > 0.0 {
            trunc((street_price - shape.fascia).max(0.0), 2)
        } else {
            0.0
        };

        if tp <= 0.0 || shape.fascia <= 0.0 {
            return KpiResult {
                npv: 0.0,
                npv_installment: 0.0,
                npv_incremental: 0.0,
                bad_debt: 0.0,
                financing_cost: 0.0,
                pb_pl: -1,
                pb_cash: -1,
                status: "MISSING_DATA".to_string(),
                status_reason: "TP o fascia non impostati".to_string(),
                target_pb,
                monthly_net: 0.0,
                cashflow: shape,
                net_arpu,
                commission,
                act_fee,
                dealer_credit_note,
            };
        }

        // Build cashflows duration-aware
        let cashflows = self.build_cashflows(&shape, tp, monthly_net);

        let npv_installment = trunc(npv(&cashflows, monthly_rate), 6);
        let npv_incremental = trunc(self.incremental_churn_npv(product, mode, mode_state), 6);
        let bad_debt = trunc(self.bad_debt_adjustment(mode, shape.fascia, shape.anticipo), 6);
        let financing_cost = trunc(
            self.financing_cost(mode, shape.fascia, shape.anticipo, shape.duration, shape.rata_hs),
            6,
        );
        let npv = trunc(npv_installment + npv_incremental - bad_debt - financing_cost, 6);
        let pb_pl = payback(&cashflows);
        let pb_cash = payback_discounted(&cashflows, monthly_rate);

        let mut reasons = Vec::new();
        if npv < 0.0 {
            reasons.push("NPV<0");
        }
        if pb_pl == -1 || pb_pl > target_pb {
            reasons.push("PB_P&L>target");
        }
        if pb_cash == -1 || pb_cash > target_pb {
            reasons.push("PB_CASH>target");
        }

        let status = if reasons.is_empty() { "PASS" } else { "ALERT" };
        let status_reason = if reasons.is_empty() {
            "OK".to_string()
        } else {
            reasons.join(", ")
        };

        KpiResult {
            npv,
            npv_installment,
            npv_incremental,
            bad_debt,
            financing_cost,
            pb_pl,
            pb_cash,
            status: status.to_string(),
            status_reason,
            target_pb,
            monthly_net,
            cashflow: shape,
            net_arpu: (net_arpu * 1000.0).round() / 1000.0,
            commission: (commission * 100.0).round() / 100.0,
            act_fee: (act_fee * 100.0).round() / 100.0,
            dealer_credit_note: (dealer_credit_note * 100.0).round() / 100.0,
        }
    }

    // ── Private methods (mirrors tiC exactly) ──────────────────

    pub fn build_cashflow_shape(&self, mode_state: &ModeState) -> CashflowShape {
        let fascia = trunc(mode_state.fascia, 6);
        let anticipo = trunc(mode_state.anticipo.max(0.0), 6);
        let importo_smart = trunc(mode_state.importo_smart.max(0.0), 6);
        let duration = match mode_state.duration {
            24 | 30 | 36 => mode_state.duration,
            _ => 30,
        };

        let mut rata_hs = trunc(mode_state.rata_hs, 6);
        let rata_smart = if mode_state.rata_smart <= 0.0 {
            trunc(importo_smart, 6)
        } else {
            trunc(mode_state.rata_smart, 6)
        };
        let sconto_mese = trunc(mode_state.sconto, 6);
        let ultima_rata = trunc(mode_state.ultima_rata, 6);

        // Fallback: derive rata_hs from montante and duration
        if rata_hs <= 0.0 && fascia > 0.0 {
            let montante = (fascia - anticipo).max(0.0);
            let ultima = if ultima_rata > 0.0 { ultima_rata } else { 0.0 };
            rata_hs = match duration {
                24 => trunc((montante - ultima).max(0.0) / 24.0, 6),
                30 => trunc((montante - ultima - importo_smart * 5.0).max(0.0) / 24.0, 6),
                _  => trunc((montante - ultima).max(0.0) / 36.0, 6),
            };
        }

        CashflowShape {
            fascia,
            anticipo,
            importo_smart,
            rata_hs,
            rata_smart,
            ultima_rata,
            sconto_mese,
            duration,
        }
    }

    pub fn build_cashflows(&self, shape: &CashflowShape, tp: f64, monthly_net: f64) -> Vec<f64> {
        let duration = match shape.duration {
            24 | 30 | 36 => shape.duration as usize,
            _ => 30,
        };
        let fascia = shape.fascia;
        let anticipo = shape.anticipo;
        let rata_hs = shape.rata_hs;
        let rata_smart = shape.rata_smart;

        let cf0 = trunc(-tp + anticipo, 6);

        match duration {
            24 => {
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    trunc((fascia - anticipo - 23.0 * rata_hs).max(0.0), 6)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(trunc(monthly_net, 6)).take(23));
                cfs.push(trunc(ultima_rata, 6));
                cfs
            }
            30 => {
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    trunc((fascia - anticipo - 24.0 * rata_hs - 5.0 * rata_smart).max(0.0), 6)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(trunc(monthly_net, 6)).take(24));
                cfs.extend(std::iter::repeat(trunc(rata_smart, 6)).take(5));
                cfs.push(trunc(ultima_rata, 6));
                cfs
            }
            _ => {
                // 36m
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    trunc((fascia - anticipo - 35.0 * rata_hs).max(0.0), 6)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(trunc(monthly_net, 6)).take(35));
                cfs.push(trunc(ultima_rata, 6));
                cfs
            }
        }
    }

    fn incremental_churn_npv(
        &self,
        product: &Product,
        mode: &str,
        mode_state: &ModeState,
    ) -> f64 {
        if self.churn_curves.is_none() {
            return 0.0;
        }
        let curves = self.churn_curves.as_ref().unwrap();
        let arpu_no_action = self.net_arpu(product);
        let sconto_tariffa = mode_state.sconto_tariffa;
        let arpu_action = arpu_no_action - sconto_tariffa / 1.22;
        let wacc = self.annual_rate(mode);

        let engine = ChurnEngine::new(&curves.action, &curves.no_action);
        engine.incremental_npv(arpu_no_action, arpu_action, wacc)
    }

    pub fn bad_debt_adjustment(&self, mode: &str, fascia: f64, anticipo: f64) -> f64 {
        let key = match lookup_str(BAD_DEBT_MODE_MAP, mode) {
            Some(k) => k,
            None => return 0.0,
        };
        let montante = fascia - anticipo;
        if montante <= 0.0 {
            return 0.0;
        }
        let rate = self.assumption(key, 0.0);
        trunc(montante * rate, 6)
    }

    pub fn financing_cost(
        &self,
        mode: &str,
        fascia: f64,
        anticipo: f64,
        duration: u32,
        rata_hs: f64,
    ) -> f64 {
        let (internal_key, customer_key) = match lookup_fin(mode) {
            Some(keys) => keys,
            None => return 0.0,
        };
        let montante = fascia - anticipo;
        if montante <= 0.0 {
            return 0.0;
        }
        let internal = self.assumption(internal_key, 0.0);
        let customer = self.assumption(customer_key, 0.0);
        let spread = internal - customer;
        if spread <= 0.0 {
            return 0.0;
        }
        let monthly_rate = trunc((1.0 + self.annual_rate(mode)).powf(1.0 / 12.0) - 1.0, 6);
        let mut remaining = montante;
        let mut total = 0.0;
        for m in 0..duration {
            let cost_m = remaining * spread / 12.0;
            total += cost_m / (1.0 + monthly_rate).powi(m as i32 + 1);
            remaining -= rata_hs;
            if remaining < 0.0 {
                remaining = 0.0;
            }
        }
        trunc(total, 6)
    }

    pub fn annual_rate(&self, mode: &str) -> f64 {
        match mode {
            "VAR_RID" => self.assumption("BAD_DEBT_RID", 0.0936),
            "FIN_COMPASS" | "RELOAD_COMP" => self.assumption("RATE_COMPASS", 0.083),
            "FIN_FINDO" | "RELOAD_FINDO" => self.assumption("RATE_FINDO", 0.0805),
            _ => self.assumption("BAD_DEBT_CC", 0.081),
        }
    }

    fn target_pb(&self, mode: &str) -> f64 {
        match mode {
            "FIN_COMPASS" | "FIN_FINDO" | "RELOAD_COMP" | "RELOAD_FINDO" => {
                self.assumption("TARGET_PB_FIN", 16.0)
            }
            _ => self.assumption("TARGET_PB_VAR", 12.0),
        }
    }

    pub fn net_arpu(&self, product: &Product) -> f64 {
        let cluster = product.cluster.trim().to_uppercase();
        let is_pk = cluster.contains("PK");
        if let Some(arpu_key) = lookup_str(PROFILE_ARPU_MAP, &cluster) {
            if is_pk {
                let pk_key = format!("{}_PK", arpu_key);
                if self.assumptions.contains_key(&pk_key) {
                    return self.assumption(&pk_key, 0.0);
                }
            }
            return self.assumption(arpu_key, 0.0);
        }
        0.0
    }

    pub fn commission(&self, mode: &str) -> f64 {
        let key = lookup_str(MODE_COMMISSION_MAP, mode).unwrap_or("COMM_VAR");
        self.assumption(key, 0.0)
    }

    pub(crate) fn assumption(&self, key: &str, default: f64) -> f64 {
        match self.assumptions.get(key) {
            Some(a) => {
                let v = a.value;
                trunc(if v != 0.0 { v } else { default }, 6)
            }
            None => default,
        }
    }
}

// ── Static KPI functions ───────────────────────────────────────

/// NPV: Σ CF_t / (1 + monthly_rate)^t
pub fn npv(cashflows: &[f64], monthly_rate: f64) -> f64 {
    cashflows
        .iter()
        .enumerate()
        .map(|(t, cf)| cf / (1.0 + monthly_rate).powi(t as i32))
        .sum()
}

/// Payback P&L: first month where cumulative sum ≥ 0.
pub fn payback(cashflows: &[f64]) -> i32 {
    let mut cumulative = 0.0;
    for (i, cf) in cashflows.iter().enumerate() {
        cumulative += cf;
        if cumulative >= 0.0 {
            return i as i32;
        }
    }
    -1
}

/// Payback Cash (discounted): first month where discounted cumulative sum ≥ 0.
pub fn payback_discounted(cashflows: &[f64], monthly_rate: f64) -> i32 {
    let mut cumulative = 0.0;
    for (i, cf) in cashflows.iter().enumerate() {
        cumulative += cf / (1.0 + monthly_rate).powi(i as i32);
        if cumulative >= 0.0 {
            return i as i32;
        }
    }
    -1
}