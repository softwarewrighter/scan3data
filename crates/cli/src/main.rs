//! scan2data CLI
//!
//! Command-line interface for processing IBM 1130 scans

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scan2data")]
#[command(about = "Process IBM 1130 punch card and listing scans", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest scanned images into a scan set
    Ingest {
        /// Input directory or file
        #[arg(short, long)]
        input: String,

        /// Output directory for scan set
        #[arg(short, long)]
        output: String,
    },

    /// Analyze a scan set and classify artifacts
    Analyze {
        /// Scan set directory
        #[arg(short, long)]
        scan_set: String,

        /// Use LLM for classification
        #[arg(long)]
        use_llm: bool,
    },

    /// Export a scan set to emulator format
    Export {
        /// Scan set directory
        #[arg(short, long)]
        scan_set: String,

        /// Output file
        #[arg(short, long)]
        output: String,

        /// Format: card_deck or listing
        #[arg(short, long, default_value = "card_deck")]
        format: String,
    },

    /// Serve the web UI
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Mode: spa (standalone) or api (with backend)
        #[arg(short, long, default_value = "spa")]
        mode: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { input, output } => {
            println!("Ingesting {} -> {}", input, output);
            // TODO: Implement ingest command
            Ok(())
        }
        Commands::Analyze { scan_set, use_llm } => {
            println!("Analyzing scan set: {} (LLM: {})", scan_set, use_llm);
            // TODO: Implement analyze command
            Ok(())
        }
        Commands::Export {
            scan_set,
            output,
            format,
        } => {
            println!("Exporting {} -> {} (format: {})", scan_set, output, format);
            // TODO: Implement export command
            Ok(())
        }
        Commands::Serve { port, mode } => {
            println!("Serving {} mode on port {}", mode, port);
            // TODO: Implement serve command
            // - For "spa" mode: serve static files
            // - For "api" mode: start REST API server
            Ok(())
        }
    }
}
