//! Workbook Compare — Cell-level diff between two xlsx files.

use calamine::{open_workbook, Reader, Xlsx, Data, Range};
use std::path::Path;
use std::collections::HashMap;

/// A single cell difference between source and exported.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CellDiff {
    pub sheet: String,
    pub cell: String,
    pub source_value: String,
    pub exported_value: String,
}

/// Summary of a workbook comparison.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareSummary {
    pub total_cells: usize,
    pub matching_cells: usize,
    pub diff_cells: usize,
    pub sheets_compared: usize,
    pub diffs: Vec<CellDiff>,
}

pub struct WorkbookCompare;

impl WorkbookCompare {
    /// Compare two xlsx files at cell level.
    pub fn compare(
        source_path: &Path,
        exported_path: &Path,
    ) -> anyhow::Result<CompareSummary> {
        let mut source: Xlsx<_> = open_workbook(source_path)?;
        let mut exported: Xlsx<_> = open_workbook(exported_path)?;

        let source_sheets: Vec<(String, Range<Data>)> = source.worksheets();
        let exported_sheets: Vec<(String, Range<Data>)> = exported.worksheets();

        // Index sheets by uppercase name for case-insensitive matching
        let source_map: HashMap<String, usize> = source_sheets.iter()
            .enumerate()
            .map(|(i, (name, _))| (name.to_uppercase(), i))
            .collect();
        let exported_map: HashMap<String, usize> = exported_sheets.iter()
            .enumerate()
            .map(|(i, (name, _))| (name.to_uppercase(), i))
            .collect();

        let mut diffs = Vec::new();
        let mut total_cells = 0usize;
        let mut matching_cells = 0usize;
        let mut sheets_compared = 0usize;

        // Compare sheets that exist in both
        for (key, &src_idx) in &source_map {
            if let Some(&exp_idx) = exported_map.get(key) {
                sheets_compared += 1;
                let (sheet_name, src_range) = &source_sheets[src_idx];
                let (_, exp_range) = &exported_sheets[exp_idx];

                // Use .get((row, col)) for indexed access on ranges
                let (src_start, src_end) = get_range_bounds(src_range);
                let (_exp_start, exp_end) = get_range_bounds(exp_range);
                let max_row = src_end.0.max(exp_end.0);
                let max_col = src_end.1.max(exp_end.1);

                for row in src_start.0..=max_row {
                    for col in src_start.1..=max_col {
                        let src_val = cell_data_to_string(src_range.get((row, col)));
                        let exp_val = cell_data_to_string(exp_range.get((row, col)));

                        // Skip empty cells in both
                        if src_val.is_empty() && exp_val.is_empty() {
                            continue;
                        }

                        total_cells += 1;

                        if values_match(&src_val, &exp_val) {
                            matching_cells += 1;
                        } else {
                            let cell_ref = format!("{}{}", col_letter(col), row + 1);
                            diffs.push(CellDiff {
                                sheet: sheet_name.clone(),
                                cell: cell_ref,
                                source_value: src_val,
                                exported_value: exp_val,
                            });
                        }
                    }
                }
            }
        }

        let diff_cells = diffs.len();

        Ok(CompareSummary {
            total_cells,
            matching_cells,
            diff_cells,
            sheets_compared,
            diffs,
        })
    }
}

/// Get (start, end) bounds of a range as ((row, col), (row, col)).
fn get_range_bounds(range: &Range<Data>) -> ((usize, usize), (usize, usize)) {
    let start = range.start().unwrap_or((0, 0));
    let end = range.end().unwrap_or((0, 0));
    (
        (start.0 as usize, start.1 as usize),
        (end.0 as usize, end.1 as usize),
    )
}

/// Convert Option<&Data> to string for comparison.
fn cell_data_to_string(cell: Option<&Data>) -> String {
    match cell {
        None | Some(Data::Empty) => String::new(),
        Some(Data::String(s)) => s.clone(),
        Some(Data::Float(f)) => {
            let rounded = (*f * 10000.0).round() / 10000.0;
            format!("{}", rounded)
        }
        Some(Data::Int(i)) => format!("{}", i),
        Some(Data::Bool(b)) => format!("{}", b),
        Some(Data::DateTime(dt)) => format!("{:.6}", dt.as_f64()),
        Some(Data::DateTimeIso(s)) => s.clone(),
        Some(Data::DurationIso(s)) => s.clone(),
        Some(Data::Error(e)) => format!("ERROR:{:?}", e),
    }
}

/// Compare two cell values with float tolerance.
fn values_match(a: &str, b: &str) -> bool {
    if let (Ok(a_f), Ok(b_f)) = (a.parse::<f64>(), b.parse::<f64>()) {
        return (a_f - b_f).abs() < 0.001;
    }
    a == b
}

/// Convert 0-indexed column to Excel column letter.
fn col_letter(col: usize) -> String {
    let mut result = String::new();
    let mut c = col;
    loop {
        result.insert(0, (b'A' + (c % 26) as u8) as char);
        c = if c < 26 { break; } else { c / 26 - 1 };
    }
    result
}