//! tiD — CVM Pricing Cockpit (Rust)
//!
//! Usage:
//!   tid serve           — Start HTTP server on port 5002
//!   tid compliance      — Run compliance check
//!   tid validate         — Validate data files

use clap::Parser;

#[derive(Parser)]
#[command(name = "tid")]
#[command(about = "tiC reimagined in Rust — CVM Pricing Cockpit")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Start HTTP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "5002")]
        port: u16,
    },
    /// Run compliance check
    Compliance {
        /// Tolerance for KPI comparison
        #[arg(short, long, default_value = "0.01")]
        tolerance: f64,
        /// Output format
        #[arg(short, long, default_value = "json")]
        report: String,
    },
    /// Validate data files
    Validate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => run_server(port).await,
        Commands::Compliance { tolerance, report } => run_compliance(tolerance, &report),
        Commands::Validate => run_validate(),
    }
}

async fn run_server(port: u16) -> anyhow::Result<()> {
    use tid::engine::pricing::PricingEngine;
    use tid::api::routes::create_router;
    use tid::paths;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    println!("================================");
    println!("tiD — CVM Pricing Cockpit");
    println!("================================");
    println!();

    tracing::info!("Base dir: {}", paths::base_dir().display());

    // Ensure data directories exist
    paths::ensure_data_dirs()?;

    // Load data
    println!("📦 Loading data...");
    let engine = PricingEngine::load()?;
    println!("✅ Loaded {} products", engine.products.len());

    let shared_engine = Arc::new(RwLock::new(engine));

    let app = create_router(shared_engine);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 Starting tiD on http://127.0.0.1:{}", port);
    println!("Press Ctrl+C to stop");
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn run_compliance(tolerance: f64, report: &str) -> anyhow::Result<()> {
    use tid::engine::pricing::PricingEngine;
    use tid::paths;

    paths::ensure_data_dirs()?;

    println!("📊 Running compliance check (tolerance={})...", tolerance);
    let engine = PricingEngine::load()?;
    let result = engine.run_compliance(tolerance);

    match report {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "md" => {
            println!("# Compliance Report");
            println!();
            println!("| Metric | Value |");
            println!("|--------|-------|");
            println!("| Score globale | {:.2}% |", result.score_globale);
            println!("| Checked | {} |", result.checked);
            println!("| Passed | {} |", result.passed);
            println!();
            if !result.mismatches.is_empty() {
                println!("## Mismatches (first 20)");
                println!("| Product | Mode | KPI | Expected | Actual | Delta |");
                println!("|---------|------|-----|----------|--------|-------|");
                for m in result.mismatches.iter().take(20) {
                    println!("| {} | {} | {} | {:.4} | {:.4} | {:.4} |",
                        m.product_id, m.mode, m.kpi, m.expected, m.actual, m.delta);
                }
            }
        }
        _ => {
            println!("Score: {:.2}% ({}/{})", result.score_globale, result.passed, result.checked);
        }
    }

    if result.score_globale >= 80.0 {
        println!("✅ PASS (≥80%)");
        std::process::exit(0);
    } else {
        println!("❌ FAIL (<80%)");
        std::process::exit(1);
    }
}

fn run_validate() -> anyhow::Result<()> {
    use tid::paths;

    paths::ensure_data_dirs()?;

    println!("🔍 Validating data files...");

    let econ = paths::resolve_input("Listino_CVM_ECONOMICS.xlsx");
    let fasce = paths::resolve_input("Listino_CVM_FASCE.xlsx");
    let output = paths::resolve_input("output_TI_CVM.xlsb")
        .or_else(|_| paths::resolve_input("output_TI_CVM.xlsx"));

    match econ {
        Ok(p) => println!("✅ ECONOMICS: {}", p.display()),
        Err(e) => println!("❌ ECONOMICS: {}", e),
    }
    match fasce {
        Ok(p) => println!("✅ FASCE: {}", p.display()),
        Err(e) => println!("❌ FASCE: {}", e),
    }
    match output {
        Ok(p) => println!("✅ OUTPUT: {}", p.display()),
        Err(e) => println!("❌ OUTPUT: {}", e),
    }

    Ok(())
}