//! Formula Engine — Excel formula string generation.
//!
//! Mirrors economics_engine calculations as formula strings
//! for live-formula xlsx exports.

use crate::engine::types::*;

pub struct FormulaEngine;

impl FormulaEngine {
    /// Convert a 0-indexed column number to an Excel column letter (A, B, … Z, AA, AB, …).
    pub fn col_letter(col: usize) -> String {
        let mut result = String::new();
        let mut c = col;
        loop {
            result.insert(0, (b'A' + (c % 26) as u8) as char);
            if c < 26 {
                break;
            }
            c = (c / 26) - 1;
        }
        result
    }

    /// WACC/annual-rate cell reference per mode, pointing into the ASSUMPTIONS sheet.
    pub fn wacc_cell(mode: &str) -> String {
        match mode {
            "VAR_RID" => "ASSUMPTIONS!B4".to_string(),
            "FIN_COMPASS" | "RELOAD_COMP" => "ASSUMPTIONS!B6".to_string(),
            "FIN_FINDO" | "RELOAD_FINDO" => "ASSUMPTIONS!B7".to_string(),
            _ => "ASSUMPTIONS!B5".to_string(), // VAR_CC, RELOAD_CC etc.
        }
    }

    /// Bad-debt rate cell reference per mode.
    pub fn bad_debt_rate_cell(mode: &str) -> String {
        match mode {
            "VAR_RID" => "ASSUMPTIONS!B10".to_string(),
            "VAR_CC" | "RELOAD_CC" | "RELOAD_COMP" | "RELOAD_FINDO" => "ASSUMPTIONS!B9".to_string(),
            _ => "0".to_string(), // FIN modes: no bad debt
        }
    }

    /// Financing internal rate cell per FIN mode.
    pub fn fin_internal_rate_cell(mode: &str) -> String {
        match mode {
            "FIN_COMPASS" => "ASSUMPTIONS!B12".to_string(),
            "FIN_FINDO" => "ASSUMPTIONS!B14".to_string(),
            _ => "0".to_string(),
        }
    }

    /// Financing customer rate cell per FIN mode.
    pub fn fin_customer_rate_cell(mode: &str) -> String {
        match mode {
            "FIN_COMPASS" => "ASSUMPTIONS!B13".to_string(),
            "FIN_FINDO" => "ASSUMPTIONS!B15".to_string(),
            _ => "0".to_string(),
        }
    }

    /// Generate NPV formula string for a product+mode.
    ///
    /// References the final discounted cumulative cell on the CASHFLOWS sheet.
    pub fn npv_formula(mode: &str, shape: &CashflowShape, cashflow_row: u32) -> String {
        let _duration = shape.duration;
        let _rate_cell = Self::wacc_cell(mode);
        // NPV = final discounted cumulative from CASHFLOWS sheet
        let final_row = cashflow_row + shape.duration as u32;
        format!("=CASHFLOWS!F{final_row}")
    }

    /// Payback P&L formula: first month where cumulative cashflow >= 0.
    ///
    /// Uses MATCH on the cumulative column (E) of the CASHFLOWS sheet.
    pub fn payback_formula(cashflow_row: u32, duration: u32) -> String {
        let start = cashflow_row;
        let end = cashflow_row + duration;
        format!("=IFERROR(MATCH(TRUE,CASHFLOWS!E{start}:E{end}>=0,0)-1,-1)")
    }

    /// Payback Cash formula: first month where discounted cumulative >= 0.
    ///
    /// References the discounted cumulative column (F) of the CASHFLOWS sheet.
    pub fn payback_cash_formula(cashflow_row: u32, duration: u32) -> String {
        let start = cashflow_row;
        let end = cashflow_row + duration;
        format!("=IFERROR(MATCH(TRUE,CASHFLOWS!F{start}:F{end}>=0,0)-1,-1)")
    }

    /// Bad debt formula: montante * bad_debt_rate
    ///
    /// montante = fascia - anticipo
    pub fn bad_debt_formula(mode: &str, fascia_cell: &str, anticipo_cell: &str) -> String {
        let rate_cell = Self::bad_debt_rate_cell(mode);
        format!("=({fascia_cell}-{anticipo_cell})*{rate_cell}")
    }

    /// Financing cost formula: sum of discounted financing costs from CASHFLOWS sheet.
    ///
    /// For non-FIN modes, returns "=0".
    pub fn financing_cost_formula(
        mode: &str,
        cashflow_row: u32,
        duration: u32,
    ) -> String {
        if !mode.starts_with("FIN") {
            return "=0".to_string();
        }
        let fin_start = cashflow_row;
        let fin_end = cashflow_row + duration;
        format!("=SUM(CASHFLOWS!I{fin_start}:I{fin_end})")
    }

    /// Net ARPU formula: VLOOKUP into the ASSUMPTIONS sheet based on cluster.
    ///
    /// The cluster→ARPU mapping is stored on the ASSUMPTIONS sheet starting at row 20+.
    /// Col A = key, Col B = value.
    pub fn net_arpu_formula(cluster_cell: &str) -> String {
        format!("=VLOOKUP({cluster_cell},ASSUMPTIONS!A20:B35,2,FALSE)")
    }
}