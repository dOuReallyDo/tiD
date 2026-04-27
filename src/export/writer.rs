//! Workbook Writer — xlsx generation.
//!
//! Uses rust_xlsxwriter for fast Excel export.
//! v0.4: exports include live Excel formulas referencing an ASSUMPTIONS sheet.

use crate::engine::types::*;
use crate::engine::formula::FormulaEngine;
use rust_xlsxwriter::*;
use std::collections::HashMap;

pub struct WorkbookWriter;

impl WorkbookWriter {
    /// Export economics KPIs to xlsx with live formulas.
    ///
    /// Sheets produced:
    ///   - VALUES: raw computed values (quick verification, unchanged)
    ///   - KPI:    live-formula KPI summary (NPV, Payback P&L, Payback Cash)
    ///   - ASSUMPTIONS: all assumption key→value pairs
    ///   - CASHFLOWS: per-product-mode cashflow detail rows
    ///   - CASHFLOW_SUMMARY: formula references to CASHFLOWS aggregation
    pub fn write_economics(
        products: &[Product],
        assumptions: &HashMap<String, Assumption>,
        output_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let mut workbook = Workbook::new();

        // ── Sheet 1: VALUES (raw computed values, unchanged) ───────────
        Self::write_values_sheet(&mut workbook, products)?;

        // ── Sheet 2: ASSUMPTIONS ──────────────────────────────────────
        Self::write_assumptions_sheet(&mut workbook, assumptions)?;

        // ── Sheet 3: CASHFLOWS ────────────────────────────────────────
        let cashflow_row_map = Self::write_cashflows_sheet(&mut workbook, products)?;

        // ── Sheet 4: KPI (with live formulas) ─────────────────────────
        Self::write_kpi_sheet(&mut workbook, products, &cashflow_row_map)?;

        // ── Sheet 5: CASHFLOW_SUMMARY ────────────────────────────────
        Self::write_cashflow_summary_sheet(&mut workbook, products, &cashflow_row_map)?;

        workbook.save(output_path)?;
        Ok(())
    }

    // ── VALUES sheet: raw computed values ────────────────────────────

    fn write_values_sheet(workbook: &mut Workbook, products: &[Product]) -> anyhow::Result<()> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("VALUES")?;

        let headers = [
            "ID", "Name", "Mode", "NPV", "Payback P&L", "Payback Cash",
            "Bad Debt", "Financing Cost", "Net ARPU", "Status",
        ];
        let bold = Format::new().set_bold();
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &bold)?;
        }

        let mut row = 1u32;
        for product in products {
            for mode in MODES {
                if let Some(mode_state) = product.modes.get(*mode) {
                    if let Some(kpis) = &mode_state.kpis {
                        worksheet.write_string(row, 0, &product.id)?;
                        worksheet.write_string(row, 1, &product.name)?;
                        worksheet.write_string(row, 2, *mode)?;
                        worksheet.write_number(row, 3, kpis.npv)?;
                        worksheet.write_number(row, 4, kpis.pb_pl as f64)?;
                        worksheet.write_number(row, 5, kpis.pb_cash as f64)?;
                        worksheet.write_number(row, 6, kpis.bad_debt)?;
                        worksheet.write_number(row, 7, kpis.financing_cost)?;
                        worksheet.write_number(row, 8, kpis.net_arpu)?;
                        worksheet.write_string(row, 9, &kpis.status)?;
                        row += 1;
                    }
                }
            }
        }
        Ok(())
    }

    // ── ASSUMPTIONS sheet ─────────────────────────────────────────────

    fn write_assumptions_sheet(
        workbook: &mut Workbook,
        assumptions: &HashMap<String, Assumption>,
    ) -> anyhow::Result<()> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("ASSUMPTIONS")?;

        let bold = Format::new().set_bold();
        worksheet.write_string_with_format(0, 0, "Key", &bold)?;
        worksheet.write_string_with_format(0, 1, "Value", &bold)?;
        worksheet.write_string_with_format(0, 2, "Label", &bold)?;

        // Sort assumptions by key for deterministic output
        let mut sorted: Vec<_> = assumptions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (i, (key, assumption)) in sorted.iter().enumerate() {
            let r = (i + 1) as u32;
            worksheet.write_string(r, 0, key.as_str())?;
            worksheet.write_number(r, 1, assumption.value)?;
            worksheet.write_string(r, 2, assumption.label.as_str())?;
        }

        Ok(())
    }

    // ── CASHFLOWS sheet ──────────────────────────────────────────────

    /// Returns a map from "product_id:mode" to the starting row in CASHFLOWS.
    fn write_cashflows_sheet(
        workbook: &mut Workbook,
        products: &[Product],
    ) -> anyhow::Result<HashMap<String, u32>> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("CASHFLOWS")?;

        let bold = Format::new().set_bold();
        let num_fmt = Format::new().set_num_format("0.00");

        // Main header
        let headers = [
            "Product ID", "Mode", "Month", "Cashflow", "Cumulative",
            "Disc. Cumulative", "Remaining Balance", "Fin. Cost/Month", "Disc. Fin. Cost",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &bold)?;
        }

        let mut row_map: HashMap<String, u32> = HashMap::new();
        let mut current_row = 1u32;

        for product in products {
            for mode in MODES {
                if let Some(mode_state) = product.modes.get(*mode) {
                    if let Some(kpis) = &mode_state.kpis {
                        let key = format!("{}:{}", product.id, mode);
                        row_map.insert(key, current_row);

                        let shape = &kpis.cashflow;

                        // Build cashflow vector
                        let monthly_net = shape.rata_hs - shape.sconto_mese;
                        let cashflows = Self::build_cashflows_for_export(shape, product.tp, monthly_net);

                        let rate_ref = FormulaEngine::wacc_cell(mode);

                        for (m, cf_value) in cashflows.iter().enumerate() {
                            let r = current_row + m as u32;
                            worksheet.write_string(r, 0, product.id.as_str())?;
                            worksheet.write_string(r, 1, *mode)?;
                            worksheet.write_number(r, 2, m as f64)?;
                            worksheet.write_number_with_format(r, 3, *cf_value, &num_fmt)?;

                            // Cumulative (E = col 4)
                            if m == 0 {
                                worksheet.write_number_with_format(r, 4, *cf_value, &num_fmt)?;
                            } else {
                                let prev = r - 1;
                                let f = format!("=E{prev}+D{r}");
                                worksheet.write_formula_with_format(
                                    r, 4, Formula::new(f.as_str()), &num_fmt,
                                )?;
                            }

                            // Discounted cumulative (F = col 5)
                            if m == 0 {
                                let f = format!("=D{r}/((1+({rate_ref})^(1/12)-1)^{m})");
                                worksheet.write_formula_with_format(
                                    r, 5, Formula::new(f.as_str()), &num_fmt,
                                )?;
                            } else {
                                let prev = r - 1;
                                let f = format!("=F{prev}+D{r}/((1+({rate_ref})^(1/12)-1)^{m})");
                                worksheet.write_formula_with_format(
                                    r, 5, Formula::new(f.as_str()), &num_fmt,
                                )?;
                            }

                            // Financing columns (G, H, I) — only for FIN modes
                            if mode.starts_with("FIN") {
                                // G: Remaining balance
                                if m == 0 {
                                    let f = format!("=({f}-{a})", f = shape.fascia, a = shape.anticipo);
                                    worksheet.write_formula_with_format(
                                        r, 6, Formula::new(f.as_str()), &num_fmt,
                                    )?;
                                } else {
                                    let prev = r - 1;
                                    let f = format!("=MAX(G{prev}-{rata},0)", rata = shape.rata_hs);
                                    worksheet.write_formula_with_format(
                                        r, 6, Formula::new(f.as_str()), &num_fmt,
                                    )?;
                                }

                                // H: Monthly financing cost = remaining * spread / 12
                                let int_rate = FormulaEngine::fin_internal_rate_cell(mode);
                                let cust_rate = FormulaEngine::fin_customer_rate_cell(mode);
                                let f = format!("=G{r}*({int_rate}-{cust_rate})/12");
                                worksheet.write_formula_with_format(
                                    r, 7, Formula::new(f.as_str()), &num_fmt,
                                )?;

                                // I: Discounted financing cost
                                let f = format!("=H{r}/((1+({rate_ref})^(1/12)-1)^{month})", month = m + 1);
                                worksheet.write_formula_with_format(
                                    r, 8, Formula::new(f.as_str()), &num_fmt,
                                )?;
                            }
                        }

                        current_row += cashflows.len() as u32;
                    }
                }
            }
        }

        Ok(row_map)
    }

    // ── KPI sheet (live formulas) ────────────────────────────────────

    fn write_kpi_sheet(
        workbook: &mut Workbook,
        products: &[Product],
        cashflow_row_map: &HashMap<String, u32>,
    ) -> anyhow::Result<()> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("KPI")?;

        let bold = Format::new().set_bold();
        let num_fmt = Format::new().set_num_format("0.00");

        // We'll use an extra column for the cluster value (VLOOKUP helper)
        let headers = [
            "ID", "Name", "Mode", "NPV (formula)", "Payback P&L (formula)",
            "Payback Cash (formula)", "Bad Debt (formula)", "Fin. Cost (formula)",
            "Net ARPU (formula)", "Status", "Cluster (helper)",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &bold)?;
        }

        let pass_fmt = Format::new().set_font_color(Color::Green).set_bold();
        let alert_fmt = Format::new().set_font_color(Color::Red).set_bold();

        let mut row = 1u32;
        for product in products {
            for mode in MODES {
                if let Some(mode_state) = product.modes.get(*mode) {
                    if let Some(kpis) = &mode_state.kpis {
                        let key = format!("{}:{}", product.id, mode);
                        let cf_start = cashflow_row_map.get(&key).copied().unwrap_or(1);
                        let duration = mode_state.duration;

                        worksheet.write_string(row, 0, product.id.as_str())?;
                        worksheet.write_string(row, 1, product.name.as_str())?;
                        worksheet.write_string(row, 2, *mode)?;

                        // NPV formula: final discounted cumulative from CASHFLOWS
                        let npv_formula = FormulaEngine::npv_formula(mode, &kpis.cashflow, cf_start);
                        worksheet.write_formula_with_format(
                            row, 3, Formula::new(npv_formula.as_str()), &num_fmt,
                        )?;

                        // Payback P&L formula
                        let pb_formula = FormulaEngine::payback_formula(cf_start, duration);
                        worksheet.write_formula_with_format(
                            row, 4, Formula::new(pb_formula.as_str()), &num_fmt,
                        )?;

                        // Payback Cash formula
                        let pb_cash_formula = FormulaEngine::payback_cash_formula(cf_start, duration);
                        worksheet.write_formula_with_format(
                            row, 5, Formula::new(pb_cash_formula.as_str()), &num_fmt,
                        )?;

                        // Bad Debt formula
                        let fascia_cell = format!("CASHFLOWS!D{cf_start}");
                        let anticipo_val = mode_state.anticipo.to_string();
                        let bd_formula = FormulaEngine::bad_debt_formula(mode, &fascia_cell, &anticipo_val);
                        worksheet.write_formula_with_format(
                            row, 6, Formula::new(bd_formula.as_str()), &num_fmt,
                        )?;

                        // Financing Cost formula — sum of discounted financing costs
                        let fin_formula = FormulaEngine::financing_cost_formula(mode, cf_start, duration);
                        worksheet.write_formula_with_format(
                            row, 7, Formula::new(fin_formula.as_str()), &num_fmt,
                        )?;

                        // Net ARPU formula — VLOOKUP using cluster helper column (K = col 10)
                        worksheet.write_string(row, 10, product.cluster.as_str())?;
                        let arpu_formula = FormulaEngine::net_arpu_formula(&format!("K{row}"));
                        worksheet.write_formula_with_format(
                            row, 8, Formula::new(arpu_formula.as_str()), &num_fmt,
                        )?;

                        // Status (computed value — derived string, not formula-able)
                        let status_fmt = if kpis.status == "PASS" { &pass_fmt } else { &alert_fmt };
                        worksheet.write_string_with_format(row, 9, kpis.status.as_str(), status_fmt)?;

                        row += 1;
                    }
                }
            }
        }

        Ok(())
    }

    // ── CASHFLOW_SUMMARY sheet ────────────────────────────────────────

    fn write_cashflow_summary_sheet(
        workbook: &mut Workbook,
        products: &[Product],
        cashflow_row_map: &HashMap<String, u32>,
    ) -> anyhow::Result<()> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("CASHFLOW_SUMMARY")?;

        let bold = Format::new().set_bold();
        let num_fmt = Format::new().set_num_format("0.00");

        let headers = [
            "ID", "Mode", "CF0 (Anticipo - TP)", "Total Cashflows",
            "NPV (Disc. Cum. Final)", "Final Cumulative",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &bold)?;
        }

        let mut row = 1u32;
        for product in products {
            for mode in MODES {
                if let Some(mode_state) = product.modes.get(*mode) {
                    if mode_state.kpis.is_some() {
                        let key = format!("{}:{}", product.id, mode);
                        let cf_start = cashflow_row_map.get(&key).copied().unwrap_or(1);
                        let duration = mode_state.duration;

                        worksheet.write_string(row, 0, product.id.as_str())?;
                        worksheet.write_string(row, 1, *mode)?;

                        // CF0 reference
                        let f = format!("=CASHFLOWS!D{cf_start}");
                        worksheet.write_formula_with_format(
                            row, 2, Formula::new(f.as_str()), &num_fmt,
                        )?;

                        // Total Cashflows = SUM of D column for this block
                        let cf_end = cf_start + duration;
                        let f = format!("=SUM(CASHFLOWS!D{cf_start}:D{cf_end})");
                        worksheet.write_formula_with_format(
                            row, 3, Formula::new(f.as_str()), &num_fmt,
                        )?;

                        // NPV = final discounted cumulative
                        let f = format!("=CASHFLOWS!F{cf_end}");
                        worksheet.write_formula_with_format(
                            row, 4, Formula::new(f.as_str()), &num_fmt,
                        )?;

                        // Final cumulative
                        let f = format!("=CASHFLOWS!E{cf_end}");
                        worksheet.write_formula_with_format(
                            row, 5, Formula::new(f.as_str()), &num_fmt,
                        )?;

                        row += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Build cashflow vector for export (mirrors economics engine logic).
    fn build_cashflows_for_export(
        shape: &CashflowShape,
        tp: f64,
        monthly_net: f64,
    ) -> Vec<f64> {
        let duration = match shape.duration {
            24 | 30 | 36 => shape.duration as usize,
            _ => 30,
        };

        let cf0 = -tp + shape.anticipo;

        match duration {
            24 => {
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    (shape.fascia - shape.anticipo - 23.0 * shape.rata_hs).max(0.0)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(monthly_net).take(23));
                cfs.push(ultima_rata);
                cfs
            }
            30 => {
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    (shape.fascia - shape.anticipo - 24.0 * shape.rata_hs - 5.0 * shape.rata_smart).max(0.0)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(monthly_net).take(24));
                cfs.extend(std::iter::repeat(shape.rata_smart).take(5));
                cfs.push(ultima_rata);
                cfs
            }
            _ => {
                let ultima_rata = if shape.ultima_rata > 0.0 {
                    shape.ultima_rata
                } else {
                    (shape.fascia - shape.anticipo - 35.0 * shape.rata_hs).max(0.0)
                };
                let mut cfs = vec![cf0];
                cfs.extend(std::iter::repeat(monthly_net).take(35));
                cfs.push(ultima_rata);
                cfs
            }
        }
    }

    /// Export fasce rules to xlsx.
    pub fn write_fasce(
        products: &[Product],
        output_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let headers = ["ID", "Mode", "Fascia", "Anticipo", "Rata HS", "Rata Smart", "Sconto"];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, *header)?;
        }

        let mut row = 1u32;
        for product in products {
            for mode in MODES {
                if let Some(ms) = product.modes.get(*mode) {
                    worksheet.write_string(row, 0, product.id.as_str())?;
                    worksheet.write_string(row, 1, *mode)?;
                    worksheet.write_number(row, 2, ms.fascia)?;
                    worksheet.write_number(row, 3, ms.anticipo)?;
                    worksheet.write_number(row, 4, ms.rata_hs)?;
                    worksheet.write_number(row, 5, ms.rata_smart)?;
                    worksheet.write_number(row, 6, ms.sconto)?;
                    row += 1;
                }
            }
        }

        workbook.save(output_path)?;
        Ok(())
    }
}