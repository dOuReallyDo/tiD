//! Churn Engine — Curve interpolation and incremental NPV.
//!
//! Port of tiC's churn_engine.py.

pub struct ChurnEngine<'a> {
    action: &'a [f64],
    no_action: &'a [f64],
}

impl<'a> ChurnEngine<'a> {
    pub fn new(action: &'a [f64], no_action: &'a [f64]) -> Self {
        Self { action, no_action }
    }

    /// Churn-weighted incremental service margin NPV over 42 months.
    ///
    /// pl_month[m] = -arpu_no_action * no_action[m] + arpu_action * action[m]
    /// incremental_npv = Σ pl_month[m] / (1 + wacc_monthly)^(m+1)
    pub fn incremental_npv(
        &self,
        arpu_no_action: f64,
        arpu_action: f64,
        wacc_annual: f64,
    ) -> f64 {
        let monthly_rate = (1.0 + wacc_annual).powf(1.0 / 12.0) - 1.0;

        let pl_months: Vec<f64> = (0..42)
            .map(|m| {
                let a = self.action.get(m).copied().unwrap_or(0.0);
                let na = self.no_action.get(m).copied().unwrap_or(0.0);
                -arpu_no_action * na + arpu_action * a
            })
            .collect();

        // Shift: month m discounted at (m+1) — matching tiC convention
        let shifted: Vec<f64> = std::iter::once(0.0)
            .chain(pl_months.into_iter())
            .collect();

        crate::engine::economics::npv(&shifted, monthly_rate)
    }
}