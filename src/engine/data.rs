//! Data Manager — Excel ingestion via calamine.
//!
//! Port of tiC's data_manager.py.
//! Reads Listino_CVM_ECONOMICS.xlsx, Listino_CVM_FASCE.xlsx, output_TI_CVM.

use crate::engine::types::*;
use crate::engine::economics::cell_as_f64;
use crate::paths;
use calamine::{open_workbook_auto, Data, Reader, Sheets};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// ── Assumption cell mapping (ASSUMPTIONS sheet) ────────────────

const ASSUMPTION_CELLS: &[(&str, &str)] = &[
    ("WACC_CC", "B5"),
    ("WACC_RID", "B4"),
    ("RATE_COMPASS", "B6"),
    ("RATE_FINDO", "B7"),
    ("BAD_DEBT_RID", "B4"),
    ("BAD_DEBT_CC", "B5"),
    ("RATE_COMPASS_CUSTOMER", "D6"),
    ("RATE_FINDO_CUSTOMER", "D7"),
    ("NA_A", "B10"),
    ("NA_B", "B11"),
    ("NA_C", "B12"),
    ("NA_D", "B13"),
    ("NA_E", "B14"),
    ("NA_CB", "B15"),
    ("NA_NT", "B16"),
    ("NA_A_PK", "B17"),
    ("NA_B_PK", "B18"),
    ("NA_C_PK", "B19"),
    ("NA_D_PK", "B20"),
    ("NA_E_PK", "B21"),
    ("NA_CB_PK", "B22"),
    ("NA_NT_PK", "B23"),
    ("COMM_VAR", "B25"),
    ("COMM_FIN", "B26"),
    ("COMM_RPLUS", "B27"),
    ("ACT_FEE", "B28"),
    ("TARGET_PB_VAR", "B30"),
    ("TARGET_PB_FIN", "B31"),
];

/// Parse Excel cell reference (e.g., "B4") → (row: 0-indexed, col: 0-indexed).
fn parse_cell(cell_ref: &str) -> (usize, usize) {
    let mut col = 0usize;
    let mut row_str = String::new();
    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            col = col * 26 + (c as usize - 'A' as usize + 1);
        } else {
            row_str.push(c);
        }
    }
    let row: usize = row_str.parse().unwrap_or(1) - 1; // 0-indexed
    (row, col - 1) // 0-indexed column
}

// ── DataManager ─────────────────────────────────────────────────

pub struct DataManager;

impl DataManager {
    /// Load all data from Excel files.
    pub fn load_all() -> anyhow::Result<(Vec<Product>, HashMap<String, Assumption>, Option<ChurnCurves>)> {
        let econ_path = paths::resolve_input("Listino_CVM_ECONOMICS_UPLOADED.xlsx")
            .or_else(|_| paths::resolve_input("Listino_CVM_ECONOMICS.xlsx"))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        let fasce_path = paths::resolve_input("Listino_CVM_FASCE_UPLOADED.xlsx")
            .or_else(|_| paths::resolve_input("Listino_CVM_FASCE.xlsx"))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let (products, assumptions, churn_curves) = Self::load_economics(&econ_path)?;
        let rules = Self::load_fasce_rules(&fasce_path)?;

        // Merge rules into product mode states
        let mut products = products;
        for product in &mut products {
            for (mode, mode_state) in product.modes.iter_mut() {
                if let Some(rule) = rules.get(mode).and_then(|r| r.get(&mode_state.fascia.to_string())) {
                    mode_state.rule = Some(rule.clone());
                }
            }
        }

        Ok((products, assumptions, churn_curves))
    }

    /// Load products and assumptions from ECONOMICS xlsx.
    fn load_economics(
        path: &Path,
    ) -> anyhow::Result<(Vec<Product>, HashMap<String, Assumption>, Option<ChurnCurves>)> {
        let mut workbook: Sheets<BufReader<File>> = open_workbook_auto(path)?;
        
        // Read assumptions from ASSUMPTIONS sheet
        let assumptions = Self::load_assumptions(&mut workbook)?;
        
        // Read churn curves from CHURN sheet
        let churn_curves = Self::load_churn(&mut workbook)?;

        // Read products from LISTINO_CVM sheet
        let col_map = mode_column_map();
        let kpi_map = kpi_column_map();
        let products = Self::load_products(&mut workbook, &col_map, &kpi_map)?;

        Ok((products, assumptions, churn_curves))
    }

    fn load_assumptions(
        workbook: &mut Sheets<BufReader<File>>,
    ) -> anyhow::Result<HashMap<String, Assumption>> {
        let mut assumptions = HashMap::new();
        
        if let Ok(range) = workbook.worksheet_range("ASSUMPTIONS") {
            for (key, cell_ref) in ASSUMPTION_CELLS {
                let (row, col) = parse_cell(cell_ref);
                if let Some(cell) = range.get((row, col)) {
                    let value = cell_as_f64(cell, 0.0);
                    assumptions.insert(
                        key.to_string(),
                        Assumption {
                            cell: cell_ref.to_string(),
                            value,
                            label: key.to_string(),
                        },
                    );
                }
            }
        }

        Ok(assumptions)
    }

    fn load_churn(workbook: &mut Sheets<BufReader<File>>) -> anyhow::Result<Option<ChurnCurves>> {
        if let Ok(range) = workbook.worksheet_range("CHURN") {
            let mut action = Vec::new();
            let mut no_action = Vec::new();
            
            // Expect columns: month | action | no_action
            for row in range.rows().skip(1) {
                if row.len() >= 3 {
                    let a = cell_as_f64(&row[1], 0.0);
                    let na = cell_as_f64(&row[2], 0.0);
                    action.push(a);
                    no_action.push(na);
                }
            }
            
            if !action.is_empty() {
                return Ok(Some(ChurnCurves { action, no_action }));
            }
        }
        Ok(None)
    }

    fn load_products(
        workbook: &mut Sheets<BufReader<File>>,
        col_map: &HashMap<&str, HashMap<&str, usize>>,
        kpi_map: &HashMap<&str, HashMap<&str, usize>>,
    ) -> anyhow::Result<Vec<Product>> {
        let range = workbook.worksheet_range("LISTINO_CVM")?;
        let mut products = Vec::new();

        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 {
                continue; // skip header
            }

            // Product identity columns (col 0 = id, col 1 = name, etc.)
            let id = row_cell_str(&row, 0, &format!("P{}", row_idx));
            let name = row_cell_str(&row, 2, "");
            let tp = cell_as_f64(row.get(3).unwrap_or(&Data::Empty), 0.0);
            let full_price = cell_as_f64(row.get(4).unwrap_or(&Data::Empty), 0.0);
            let cluster = row_cell_str(&row, 5, "");
            let profile = row_cell_str(&row, 6, "");

            // Mode durations (simplified — normally from FASCE, default 30)
            let duration: u32 = 30;

            let mut modes = HashMap::new();
            for mode in MODES {
                let m_cm = match col_map.get(mode) {
                    Some(m) => m,
                    None => continue,
                };
                let k_cm = kpi_map.get(mode);

                let get_col = |key: &str| -> f64 {
                    m_cm.get(key).map_or(0.0, |&col| {
                        row.get(col).map_or(0.0, |c| cell_as_f64(c, 0.0))
                    })
                };

                let status = m_cm
                    .get("status")
                    .map_or(String::new(), |&col| row_cell_str(&row, col, ""));

                let fascia = get_col("fascia");
                let anticipo = get_col("anticipo").max(0.0);
                let importo_smart = get_col("importo_smart").max(0.0);
                let sconto_tariffa = get_col("sconto_tariffa");
                let rata_hs = get_col("rata_hs");
                let rata_smart = get_col("rata_smart");
                let ultima_rata = get_col("ultima_rata");

                // Excel baseline KPIs
                let excel_npv = k_cm.and_then(|km| {
                    km.get("npv").map(|&col| {
                        row.get(col).map_or(None, |c| {
                            let v = cell_as_f64(c, f64::NAN);
                            if v.is_nan() { None } else { Some(v) }
                        })
                    })
                }).flatten();

                let excel_pb_pl = k_cm.and_then(|km| {
                    km.get("pb_pl").map(|&col| {
                        row.get(col).map_or(None, |c| {
                            let v = cell_as_f64(c, f64::NAN);
                            if v.is_nan() { None } else { Some(v) }
                        })
                    })
                }).flatten();

                let excel_pb_cash = k_cm.and_then(|km| {
                    km.get("pb_cash").map(|&col| {
                        row.get(col).map_or(None, |c| {
                            let v = cell_as_f64(c, f64::NAN);
                            if v.is_nan() { None } else { Some(v) }
                        })
                    })
                }).flatten();

                let sconto = 0.0; // Will be filled from FASCE rules

                modes.insert(
                    mode.to_string(),
                    ModeState {
                        status,
                        fascia,
                        anticipo,
                        importo_smart,
                        sconto_tariffa,
                        rata_hs,
                        rata_smart,
                        ultima_rata,
                        sconto,
                        duration,
                        kpis: None,
                        excel_kpis: ExcelKpis {
                            npv: excel_npv,
                            pb_pl: excel_pb_pl,
                            pb_cash: excel_pb_cash,
                        },
                        rule: None,
                    },
                );
            }

            products.push(Product {
                id,
                name,
                tp,
                full_price,
                cluster,
                profile,
                duration,
                modes,
            });
        }

        Ok(products)
    }

    /// Load fasce rules from FASCE xlsx.
    fn load_fasce_rules(
        path: &Path,
    ) -> anyhow::Result<HashMap<String, HashMap<String, FasceRule>>> {
        let mut workbook: Sheets<_> = open_workbook_auto(path)?;
        let range = workbook.worksheet_range("FASCE_PIVOT")?;
        
        let mut rules: HashMap<String, HashMap<String, FasceRule>> = HashMap::new();

        // FASCE_PIVOT columns: mode, fascia, status, anticipo, rata_hs, rata_smart, sconto, mdp, chiave
        // Column positions from RULE_COLUMN_MAP (simplified initial version)
        const RULE_COLS: &[(&str, usize)] = &[
            ("mode", 0), ("fascia", 1), ("status", 2),
            ("anticipo", 3), ("rata_hs", 4), ("rata_smart", 5),
            ("sconto", 6), ("mdp", 7), ("chiave", 8),
        ];

        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 { continue; }

            let mode = row_cell_str(&row, 0, "");
            let fascia = cell_as_f64(row.get(1).unwrap_or(&Data::Empty), 0.0);
            let status = row_cell_str(&row, 2, "");
            let anticipo = cell_as_f64(row.get(3).unwrap_or(&Data::Empty), 0.0);
            let rata_hs = cell_as_f64(row.get(4).unwrap_or(&Data::Empty), 0.0);
            let rata_smart = cell_as_f64(row.get(5).unwrap_or(&Data::Empty), 0.0);
            let sconto = cell_as_f64(row.get(6).unwrap_or(&Data::Empty), 0.0);
            let mdp = cell_as_f64(row.get(7).unwrap_or(&Data::Empty), 0.0);
            let chiave = row_cell_str(&row, 8, "");

            let rule = FasceRule {
                mode: mode.clone(),
                fascia,
                status,
                anticipo,
                rata_hs,
                rata_smart,
                sconto,
                mdp,
                chiave,
            };

            rules
                .entry(mode)
                .or_insert_with(HashMap::new)
                .insert(fascia.to_string(), rule);
        }

        Ok(rules)
    }
}

/// Helper: get string value from a row cell.
fn row_cell_str(row: &[Data], col: usize, default: &str) -> String {
    match row.get(col) {
        Some(Data::String(s)) => s.clone(),
        Some(Data::Int(i)) => i.to_string(),
        Some(Data::Float(f)) => f.to_string(),
        _ => default.to_string(),
    }
}