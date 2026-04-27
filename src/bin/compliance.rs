//! tiD Compliance CLI — standalone binary for CI/testing.

use tid::engine::pricing::PricingEngine;
use tid::paths;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let tolerance = args.iter()
        .position(|a| a == "--tolerance")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.01);

    paths::ensure_data_dirs()?;

    let engine = PricingEngine::load()?;
    let result = engine.run_compliance(tolerance);

    println!("Score: {:.2}% ({}/{})", result.score_globale, result.passed, result.checked);

    if result.score_globale >= 80.0 {
        println!("✅ PASS");
        std::process::exit(0);
    } else {
        println!("❌ FAIL");
        std::process::exit(1);
    }
}