//! Formula Engine — Excel formula string generation.
//!
//! Mirrors economics_engine calculations as formula strings
//! for live-formula xlsx exports.

use crate::engine::types::*;

pub struct FormulaEngine;

impl FormulaEngine {
    /// Generate NPV formula string for a product+mode.
    pub fn npv_formula(mode: &str, shape: &CashflowShape) -> String {
        let duration = shape.duration;
        let rate_cell = Self::wacc_cell(mode);
        
        match duration {
            24 => format!(
                "NPV((1+{rate})^(1/12)-1, CF0, CF1:CF23, CF24)",
                rate = rate_cell
            ),
            30 => format!(
                "NPV((1+{rate})^(1/12)-1, CF0, CF1:CF24, CF25:CF29, CF30)",
                rate = rate_cell
            ),
            _ => format!(
                "NPV((1+{rate})^(1/12)-1, CF0, CF1:CF35, CF36)",
                rate = rate_cell
            ),
        }
    }

    /// Payback formula as COUNTIF equivalent.
    pub fn payback_formula() -> String {
        "MATCH(TRUE, CUMULATIVE>=0, 0)".to_string()
    }

    /// WACC cell reference per mode.
    fn wacc_cell(mode: &str) -> &'static str {
        match mode {
            "VAR_RID" => "ASSUMPTIONS!B4",
            "FIN_COMPASS" | "RELOAD_COMP" => "ASSUMPTIONS!B6",
            "FIN_FINDO" | "RELOAD_FINDO" => "ASSUMPTIONS!B7",
            _ => "ASSUMPTIONS!B5",
        }
    }
}