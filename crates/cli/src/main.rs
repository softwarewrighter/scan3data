//! scan3data CLI
//!
//! Command-line interface for processing IBM 1130 scans
//! The "3" represents our three-phase pipeline: Scan, Classify & Correct, Convert
//!
//! Copyright (c) 2025 Michael A Wright

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use core_pipeline::ocr::extract_text_tesseract;
use core_pipeline::preprocess::{
    compute_image_hash, detect_duplicates, preprocess_image, RgbImage,
};
use core_pipeline::types::{PageArtifact, PageId, PageMetadata, ScanSetId, ScanSetManifest};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "scan3data")]
#[command(about = "Three-phase pipeline: Scan -> Classify & Correct -> Convert", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Phase 1: Scan - Ingest scanned images into a scan set
    Ingest {
        /// Input directory or file
        #[arg(short, long)]
        input: String,

        /// Output directory for scan set
        #[arg(short, long)]
        output: String,
    },

    /// Phase 2: Classify & Correct - Analyze a scan set and classify artifacts
    Analyze {
        /// Scan set directory
        #[arg(short, long)]
        scan_set: String,

        /// Use LLM for classification
        #[arg(long)]
        use_llm: bool,
    },

    /// Phase 3: Convert - Export a scan set to emulator format
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

/// Check if a file is a supported image format
fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_lower.as_str(),
            "jpg" | "jpeg" | "png" | "tif" | "tiff" | "bmp"
        )
    } else {
        false
    }
}

/// Collect all image files from input path (file or directory)
fn collect_image_files(input_path: &str) -> Result<Vec<PathBuf>> {
    let path = Path::new(input_path);

    if !path.exists() {
        anyhow::bail!("Input path does not exist: {}", input_path);
    }

    let mut image_files = Vec::new();

    if path.is_file() {
        if is_supported_image(path) {
            image_files.push(path.to_path_buf());
        } else {
            anyhow::bail!("File is not a supported image format: {}", input_path);
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() && is_supported_image(entry_path) {
                image_files.push(entry_path.to_path_buf());
            }
        }
    } else {
        anyhow::bail!("Input path is neither a file nor directory: {}", input_path);
    }

    if image_files.is_empty() {
        anyhow::bail!("No supported image files found in: {}", input_path);
    }

    Ok(image_files)
}

/// Ingest images into a new scan set
fn ingest_scan_set(input_path: &str, output_dir: &str) -> Result<()> {
    println!("🔍 Scanning for images in: {}", input_path);

    // Collect all image files
    let image_files = collect_image_files(input_path)?;
    println!("📁 Found {} image file(s)", image_files.len());

    // Load images and compute hashes
    println!("🔢 Computing hashes for duplicate detection...");
    let mut images_with_data: Vec<(PathBuf, RgbImage)> = Vec::new();

    for (idx, file_path) in image_files.iter().enumerate() {
        print!("\r   Processing {}/{}", idx + 1, image_files.len());
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let img = image::open(file_path)
            .with_context(|| format!("Failed to load image: {}", file_path.display()))?;
        let rgb_img = img.to_rgb8();
        images_with_data.push((file_path.clone(), rgb_img));
    }
    println!();

    // Detect duplicates
    let duplicate_groups = detect_duplicates(&images_with_data);
    let unique_count = duplicate_groups.len();
    let duplicate_count = image_files.len() - unique_count;

    println!("✨ Found {} unique image(s)", unique_count);
    if duplicate_count > 0 {
        println!("   ({} duplicate(s) detected)", duplicate_count);
    }

    // Create scan set directory structure
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    let images_dir = output_path.join("images");
    let processed_dir = output_path.join("processed");
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&processed_dir)?;

    println!("📦 Creating scan set in: {}", output_dir);

    // Generate scan set ID and manifest
    let scan_set_id = ScanSetId::new();
    let created_at = Utc::now().to_rfc3339();

    let manifest = ScanSetManifest {
        scan_set_id,
        name: Path::new(input_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("scan_set")
            .to_string(),
        created_at: created_at.clone(),
        image_count: unique_count,
        original_file_count: image_files.len(),
        duplicate_count,
    };

    // Save images and create artifacts
    let mut artifacts: Vec<PageArtifact> = Vec::new();

    for (idx, group) in duplicate_groups.iter().enumerate() {
        print!("\r💾 Saving images {}/{}", idx + 1, unique_count);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Save image with hash as filename
        let image_filename = format!("{}.jpg", &group.hash[..16]); // Use first 16 chars
        let image_dest = images_dir.join(&image_filename);

        // Find the image data for this hash
        let source_image = images_with_data
            .iter()
            .find(|(_path, img)| {
                let hash = compute_image_hash(img);
                hash == group.hash
            })
            .expect("Image data not found for hash");

        // Save the image
        image::save_buffer(
            &image_dest,
            source_image.1.as_raw(),
            source_image.1.width(),
            source_image.1.height(),
            image::ColorType::Rgb8,
        )?;

        // Create artifact
        let artifact = PageArtifact {
            id: PageId::new(),
            scan_set: scan_set_id,
            raw_image_path: PathBuf::from("images").join(&image_filename),
            processed_image_path: None,
            layout_label: core_pipeline::types::ArtifactKind::Unknown,
            content_text: None,
            metadata: PageMetadata {
                content_hash: group.hash.clone(),
                original_filenames: group
                    .filenames
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
                page_number: None,
                header: None,
                footer: None,
                notes: Vec::new(),
                confidence: 0.0,
            },
        };

        artifacts.push(artifact);
    }
    println!();

    // Write manifest.json
    let manifest_path = output_path.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    // Write artifacts.json
    let artifacts_path = output_path.join("artifacts.json");
    let artifacts_json = serde_json::to_string_pretty(&artifacts)?;
    fs::write(&artifacts_path, artifacts_json)
        .with_context(|| format!("Failed to write artifacts: {}", artifacts_path.display()))?;

    println!("✅ Scan set created successfully!");
    println!("   Scan Set ID: {}", scan_set_id.0);
    println!("   Manifest: {}", manifest_path.display());
    println!("   Artifacts: {} page(s)", artifacts.len());

    Ok(())
}

/// Analyze a scan set using OCR and optional LLM classification
fn analyze_scan_set(scan_set_dir: &str, use_llm: bool) -> Result<()> {
    let scan_set_path = Path::new(scan_set_dir);

    if !scan_set_path.exists() {
        anyhow::bail!("Scan set directory does not exist: {}", scan_set_dir);
    }

    println!("🔬 Analyzing scan set: {}", scan_set_dir);

    // Load manifest
    let manifest_path = scan_set_path.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    let manifest: ScanSetManifest =
        serde_json::from_str(&manifest_json).context("Failed to parse manifest.json")?;

    println!("📋 Scan Set ID: {}", manifest.scan_set_id.0);
    println!("   Images: {}", manifest.image_count);

    // Load artifacts
    let artifacts_path = scan_set_path.join("artifacts.json");
    let artifacts_json = fs::read_to_string(&artifacts_path)
        .with_context(|| format!("Failed to read artifacts: {}", artifacts_path.display()))?;
    let mut artifacts: Vec<PageArtifact> =
        serde_json::from_str(&artifacts_json).context("Failed to parse artifacts.json")?;

    println!("📄 Processing {} artifact(s)...", artifacts.len());

    if use_llm {
        println!("🤖 LLM mode enabled (not yet implemented)");
    }

    // Process each artifact
    let processed_dir = scan_set_path.join("processed");
    let total_artifacts = artifacts.len();

    for (idx, artifact) in artifacts.iter_mut().enumerate() {
        print!("\r   Artifact {}/{}", idx + 1, total_artifacts);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Load the raw image
        let raw_image_path = scan_set_path.join(&artifact.raw_image_path);
        let img = image::open(&raw_image_path)
            .with_context(|| format!("Failed to load image: {}", raw_image_path.display()))?;

        // Preprocess the image
        let preprocessed = preprocess_image(&img)?;

        // Save preprocessed image
        let processed_filename = raw_image_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid image path"))?;
        let processed_path = processed_dir.join(processed_filename);
        preprocessed.save(&processed_path)?;

        // Update artifact with processed image path
        artifact.processed_image_path = Some(PathBuf::from("processed").join(processed_filename));

        // Run OCR
        match extract_text_tesseract(&preprocessed) {
            Ok(text) => {
                artifact.content_text = Some(text);
            }
            Err(e) => {
                // Log OCR error but continue processing
                eprintln!(
                    "\n   Warning: OCR failed for {}: {}",
                    artifact.raw_image_path.display(),
                    e
                );
                artifact.metadata.notes.push(format!("OCR failed: {}", e));
            }
        }

        // Basic classification (non-LLM baseline)
        // TODO: Add more sophisticated heuristics
        if let Some(ref text) = artifact.content_text {
            if text.len() > 100 {
                artifact.layout_label = core_pipeline::types::ArtifactKind::ListingSource;
                artifact.metadata.confidence = 0.5; // Low confidence for basic heuristic
            }
        }
    }
    println!();

    // Save updated artifacts
    let updated_artifacts_json = serde_json::to_string_pretty(&artifacts)?;
    fs::write(&artifacts_path, updated_artifacts_json)
        .with_context(|| format!("Failed to write artifacts: {}", artifacts_path.display()))?;

    println!("✅ Analysis complete!");
    println!("   Processed images: {}", processed_dir.display());
    println!("   Updated artifacts: {}", artifacts_path.display());

    // Show OCR statistics
    let with_text = artifacts
        .iter()
        .filter(|a| a.content_text.is_some())
        .count();
    let avg_text_len = artifacts
        .iter()
        .filter_map(|a| a.content_text.as_ref())
        .map(|t| t.len())
        .sum::<usize>() as f64
        / with_text.max(1) as f64;

    println!("📊 OCR Statistics:");
    println!("   Artifacts with text: {}/{}", with_text, artifacts.len());
    println!("   Average text length: {:.0} chars", avg_text_len);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { input, output } => {
            ingest_scan_set(&input, &output)?;
            Ok(())
        }
        Commands::Analyze { scan_set, use_llm } => {
            analyze_scan_set(&scan_set, use_llm)?;
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
