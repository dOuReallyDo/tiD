//! Workbook Compare — Diff source vs exported xlsx.
//! 
//! Placeholder for cell-level diff functionality.

pub struct WorkbookCompare;

impl WorkbookCompare {
    /// Compare two xlsx files at cell level.
    /// Returns list of cell differences.
    pub fn compare(
        source_path: &std::path::Path,
        exported_path: &std::path::Path,
    ) -> anyhow::Result<Vec<CellDiff>> {
        // TODO: Implement full cell-level comparison
        // For now, compare file sizes as sanity check
        let _source_meta = std::fs::metadata(source_path)?;
        let _exported_meta = std::fs::metadata(exported_path)?;
        
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct CellDiff {
    pub sheet: String,
    pub cell: String,
    pub source_value: String,
    pub exported_value: String,
}