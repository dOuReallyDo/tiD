//! Workbook Writer — xlsx generation.
//!
//! Uses rust_xlsxwriter for fast Excel export.

use crate::engine::types::*;
use rust_xlsxwriter::*;

pub struct WorkbookWriter;

impl WorkbookWriter {
    /// Export economics KPIs to xlsx.
    pub fn write_economics(products: &[Product], output_path: &std::path::Path) -> anyhow::Result<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Header
        let headers = ["ID", "Name", "Mode", "NPV", "Payback P&L", "Payback Cash", "Status"];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, *header)?;
        }

        // Bold header format
        let bold = Format::new().set_bold();
        for col in 0..headers.len() {
            worksheet.set_cell_format(0, col as u16, &bold)?;
        }

        // Data rows
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
                        worksheet.write_string(row, 6, &kpis.status)?;
                        row += 1;
                    }
                }
            }
        }

        workbook.save(output_path)?;
        Ok(())
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
                    worksheet.write_string(row, 0, &product.id)?;
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