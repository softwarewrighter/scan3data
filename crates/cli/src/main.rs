//! scan3data CLI
//!
//! Command-line interface for processing IBM 1130 scans
//! The "3" represents our three-phase pipeline: Scan, Classify & Correct, Convert
//!
//! Copyright (c) 2025 Michael A Wright

// Include build-time information
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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
#[command(version = concat!(
    env!("CARGO_PKG_VERSION"), "\n",
    "Copyright: Copyright (c) 2025 ", env!("CARGO_PKG_AUTHORS"), "\n",
    "License: MIT\n",
    "Repository: https://github.com/softwarewrighter/scan3data\n",
    "Build Host: ", env!("BUILT_HOST"), "\n",
    "Build Commit: ", env!("BUILT_GIT_COMMIT_HASH"), "\n",
    "Build Time: ", env!("BUILT_TIME_UTC")
))]
#[command(about = "Three-phase pipeline: Scan -> Classify & Correct -> Convert")]
#[command(long_about = r#"scan3data - IBM 1130 Scan Processing Pipeline

Process scanned images of IBM 1130 punch cards and computer listings into
structured data for emulator consumption.

The "3" represents our three-phase pipeline:
  1. Scan - Ingest and digitize (image acquisition, duplicate detection)
  2. Classify & Correct - Analyze and refine (OCR, LLM classification)
  3. Convert - Transform to structured output (emulator formats)

EXAMPLES:
  # Phase 1: Ingest scans
  scan3data ingest -i ./scans -o ./my_scan_set

  # Phase 2: Analyze with vision correction
  scan3data analyze -s ./my_scan_set --use-vision --vision-model llama3.2-vision:11b

  # Export raw OCR text for inspection
  scan3data text-dump -s ./my_scan_set -o output.txt

  # Generate comparison HTML (original vs corrected)
  scan3data compare -s ./my_scan_set -o comparison.html

  # Phase 3: Export to emulator format
  scan3data export -s ./my_scan_set -o deck.json -f card_deck

  # Serve web UI
  scan3data serve --mode spa --port 8080

AI CODING AGENT INSTRUCTIONS:

This CLI provides a three-phase pipeline for processing IBM 1130 scans:

PHASE 1 - INGEST:
  Use the 'ingest' command to import scanned images. This command:
  - Detects duplicate images via SHA-256 hashing
  - Stores one copy of each unique image
  - Preserves all filenames in metadata for context
  - Creates a scan set directory with artifacts.json manifest

PHASE 2 - ANALYZE:
  Use the 'analyze' command to process the scan set. Options:
  - Default: Tesseract OCR with IBM 1130 character whitelist
  - --use-vision: Apply Ollama vision model for OCR correction
  - --vision-model: Specify model (llama3.2-vision:11b recommended)
  Vision correction preserves column layout and fixes character errors

PHASE 3 - EXPORT:
  Use the 'export' command to generate emulator-ready output:
  - Format: card_deck (punch cards) or listing (printed output)
  - Output: JSON file for IBM 1130 emulator consumption

UTILITY COMMANDS:
  - text-dump: Export raw OCR text for manual inspection
  - compare: Generate HTML with side-by-side image/text comparison
  - serve: Start web UI (SPA mode or API mode)

ENVIRONMENT VARIABLES:
  GEMINI_API_KEY - Required for image cleaning (Gemini 2.5 Flash Image)
  - Get key at: https://ai.google.dev/
  - Cost: $0.039 per image

  Ollama - Optional for vision correction (local, free)
  - Install from: https://ollama.com/
  - Runs at http://localhost:11434

For more information, see: https://github.com/softwarewrighter/scan3data
"#)]
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

        /// Use vision model for OCR correction with layout preservation
        #[arg(long)]
        use_vision: bool,

        /// Vision model to use (default: llava:latest)
        #[arg(long, default_value = "llava:latest")]
        vision_model: String,
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

    /// Export raw OCR text to a text file for inspection
    TextDump {
        /// Scan set directory
        #[arg(short, long)]
        scan_set: String,

        /// Output text file
        #[arg(short, long)]
        output: String,
    },

    /// Generate HTML comparison view (original image vs corrected text)
    Compare {
        /// Scan set directory
        #[arg(short, long)]
        scan_set: String,

        /// Output HTML file
        #[arg(short, long)]
        output: String,

        /// Show column grid overlay
        #[arg(long)]
        show_grid: bool,
    },

    /// Serve the web UI
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "7214")]
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
async fn analyze_scan_set(
    scan_set_dir: &str,
    use_llm: bool,
    use_vision: bool,
    vision_model: &str,
) -> Result<()> {
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

    // Initialize vision model if requested
    let vision_client = if use_vision {
        println!("👁️  Vision mode enabled (model: {})", vision_model);
        let client = llm_bridge::OllamaClient::default_client()?;
        Some(llm_bridge::VisionModel::new(
            client,
            vision_model.to_string(),
        ))
    } else {
        None
    };

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
                // If vision correction is enabled, correct the OCR text
                if let Some(ref vision) = vision_client {
                    // Load original image bytes for vision model
                    let image_bytes = fs::read(&raw_image_path)?;

                    match vision.correct_ocr_with_layout(&image_bytes, &text).await {
                        Ok(corrected_text) => {
                            artifact.content_text = Some(corrected_text);
                            artifact
                                .metadata
                                .notes
                                .push("Vision-corrected OCR".to_string());
                        }
                        Err(e) => {
                            eprintln!(
                                "\n   Warning: Vision correction failed for {}: {}",
                                artifact.raw_image_path.display(),
                                e
                            );
                            // Fall back to raw OCR text
                            artifact.content_text = Some(text);
                            artifact
                                .metadata
                                .notes
                                .push(format!("Vision correction failed: {}", e));
                        }
                    }
                } else {
                    artifact.content_text = Some(text);
                }
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

/// Export raw OCR text to a text file for inspection
fn text_dump_scan_set(scan_set_dir: &str, output_file: &str) -> Result<()> {
    let scan_set_path = Path::new(scan_set_dir);

    if !scan_set_path.exists() {
        anyhow::bail!("Scan set directory does not exist: {}", scan_set_dir);
    }

    println!("📝 Dumping OCR text from: {}", scan_set_dir);

    // Load manifest
    let manifest_path = scan_set_path.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    let manifest: ScanSetManifest =
        serde_json::from_str(&manifest_json).context("Failed to parse manifest.json")?;

    // Load artifacts
    let artifacts_path = scan_set_path.join("artifacts.json");
    let artifacts_json = fs::read_to_string(&artifacts_path)
        .with_context(|| format!("Failed to read artifacts: {}", artifacts_path.display()))?;
    let artifacts: Vec<PageArtifact> =
        serde_json::from_str(&artifacts_json).context("Failed to parse artifacts.json")?;

    // Build output text
    let mut output = String::new();

    // Header
    output.push_str(
        "================================================================================\n",
    );
    output.push_str("SCAN SET OCR TEXT DUMP\n");
    output.push_str(&format!("Scan Set ID: {}\n", manifest.scan_set_id.0));
    output.push_str(&format!("Name: {}\n", manifest.name));
    output.push_str(&format!("Created: {}\n", manifest.created_at));
    output.push_str(&format!(
        "Images: {} unique ({} total, {} duplicates)\n",
        manifest.image_count, manifest.original_file_count, manifest.duplicate_count
    ));
    output.push_str(
        "================================================================================\n\n",
    );

    // Process each artifact
    let mut artifacts_with_text = 0;
    let mut total_chars = 0;

    for (idx, artifact) in artifacts.iter().enumerate() {
        output.push_str(
            "================================================================================\n",
        );
        output.push_str(&format!("ARTIFACT {}/{}\n", idx + 1, artifacts.len()));
        output.push_str(
            "================================================================================\n",
        );
        output.push_str(&format!("ID: {}\n", artifact.id.0));
        output.push_str(&format!("Image: {}\n", artifact.raw_image_path.display()));

        if let Some(ref processed) = artifact.processed_image_path {
            output.push_str(&format!("Processed: {}\n", processed.display()));
        }

        output.push_str(&format!("Classification: {:?}\n", artifact.layout_label));
        output.push_str(&format!("Confidence: {}\n", artifact.metadata.confidence));

        // Show original filenames if available
        if !artifact.metadata.original_filenames.is_empty() {
            output.push_str("Original Files:\n");
            for filename in &artifact.metadata.original_filenames {
                output.push_str(&format!("  - {}\n", filename));
            }
        }

        output.push_str(
            "--------------------------------------------------------------------------------\n",
        );

        if let Some(ref text) = artifact.content_text {
            output.push_str("OCR TEXT:\n");
            output.push_str("--------------------------------------------------------------------------------\n");
            output.push_str(text);
            if !text.ends_with('\n') {
                output.push('\n');
            }
            artifacts_with_text += 1;
            total_chars += text.len();
        } else {
            output.push_str("(No OCR text available)\n");
        }

        output.push_str(
            "================================================================================\n\n",
        );
    }

    // Summary footer
    output.push_str(
        "================================================================================\n",
    );
    output.push_str("SUMMARY\n");
    output.push_str(
        "================================================================================\n",
    );
    output.push_str(&format!("Total artifacts: {}\n", artifacts.len()));
    output.push_str(&format!("Artifacts with text: {}\n", artifacts_with_text));
    output.push_str(&format!("Total characters: {}\n", total_chars));
    if artifacts_with_text > 0 {
        output.push_str(&format!(
            "Average characters per artifact: {}\n",
            total_chars / artifacts_with_text
        ));
    }
    output.push_str(
        "================================================================================\n",
    );

    // Write to file
    fs::write(output_file, &output)
        .with_context(|| format!("Failed to write output file: {}", output_file))?;

    println!("✅ Text dump complete!");
    println!("   Output: {}", output_file);
    println!(
        "   Artifacts with text: {}/{}",
        artifacts_with_text,
        artifacts.len()
    );
    println!("   Total characters: {}", total_chars);
    println!("\n💡 Tip: View with a monospace font to see OCR layout");

    Ok(())
}

/// Generate HTML comparison view of original images vs corrected OCR text
fn generate_comparison_html(scan_set_dir: &str, output_file: &str, show_grid: bool) -> Result<()> {
    let scan_set_path = Path::new(scan_set_dir);

    if !scan_set_path.exists() {
        anyhow::bail!("Scan set directory does not exist: {}", scan_set_dir);
    }

    println!("📊 Generating comparison view: {}", scan_set_dir);

    // Load manifest and artifacts
    let manifest_path = scan_set_path.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    let _manifest: ScanSetManifest =
        serde_json::from_str(&manifest_json).context("Failed to parse manifest.json")?;

    let artifacts_path = scan_set_path.join("artifacts.json");
    let artifacts_json = fs::read_to_string(&artifacts_path)
        .with_context(|| format!("Failed to read artifacts: {}", artifacts_path.display()))?;
    let artifacts: Vec<PageArtifact> =
        serde_json::from_str(&artifacts_json).context("Failed to parse artifacts.json")?;

    println!("📄 Processing {} artifact(s)...", artifacts.len());

    // Build HTML
    let mut html = String::new();

    // HTML header with CSS
    html.push_str(&generate_html_header(show_grid));

    // Add each artifact comparison
    for (idx, artifact) in artifacts.iter().enumerate() {
        println!("   Artifact {}/{}", idx + 1, artifacts.len());

        // Encode image as base64 data URL
        let image_path = scan_set_path.join(&artifact.raw_image_path);
        let image_bytes = fs::read(&image_path)
            .with_context(|| format!("Failed to read image: {}", image_path.display()))?;
        let image_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_bytes);
        let image_ext = image_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let data_url = format!("data:image/{};base64,{}", image_ext, image_b64);

        // Get corrected text
        let corrected_text = artifact
            .content_text
            .as_deref()
            .unwrap_or("[No text extracted]");

        // Get metadata
        let filenames = artifact.metadata.original_filenames.join(", ");
        let notes = if artifact.metadata.notes.is_empty() {
            "None".to_string()
        } else {
            artifact.metadata.notes.join("; ")
        };

        // Add comparison section
        html.push_str(&format!(
            r#"
<div class="comparison">
    <div class="header">
        <h2>Artifact {}/{}</h2>
        <div class="metadata">
            <div><strong>Original files:</strong> {}</div>
            <div><strong>Processing notes:</strong> {}</div>
        </div>
    </div>
    <div class="side-by-side">
        <div class="panel">
            <h3>Original Scan</h3>
            <div class="image-container">
                <img src="{}" alt="Original scan" />
            </div>
        </div>
        <div class="panel">
            <h3>Corrected OCR Text</h3>
            <div class="text-container">
                <pre class="ocr-text">{}</pre>
            </div>
        </div>
    </div>
</div>
"#,
            idx + 1,
            artifacts.len(),
            html_escape(&filenames),
            html_escape(&notes),
            data_url,
            html_escape(corrected_text)
        ));
    }

    // HTML footer
    html.push_str("</body></html>");

    // Write HTML file
    fs::write(output_file, &html)
        .with_context(|| format!("Failed to write HTML file: {}", output_file))?;

    println!("✅ Comparison view complete!");
    println!("   Output: {}", output_file);
    println!("   Artifacts: {}", artifacts.len());
    println!("\n💡 Open {} in a browser to view", output_file);

    Ok(())
}

/// Generate HTML header with CSS styling
fn generate_html_header(show_grid: bool) -> String {
    let grid_css = if show_grid {
        r#"
        .ocr-text {
            background-image: repeating-linear-gradient(
                to right,
                transparent,
                transparent 0.6ch,
                rgba(0, 150, 255, 0.1) 0.6ch,
                rgba(0, 150, 255, 0.1) 0.61ch
            );
        }
        "#
    } else {
        ""
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OCR Comparison View</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #f5f5f5;
            padding: 20px;
        }}
        .comparison {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 30px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .header {{
            margin-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
            padding-bottom: 15px;
        }}
        .header h2 {{
            color: #333;
            margin-bottom: 10px;
        }}
        .metadata {{
            font-size: 14px;
            color: #666;
        }}
        .metadata div {{
            margin: 5px 0;
        }}
        .side-by-side {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
        }}
        .panel {{
            border: 1px solid #ddd;
            border-radius: 4px;
            overflow: hidden;
        }}
        .panel h3 {{
            background: #f8f8f8;
            padding: 10px 15px;
            margin: 0;
            font-size: 16px;
            color: #555;
            border-bottom: 1px solid #ddd;
        }}
        .image-container {{
            padding: 15px;
            background: #fafafa;
            display: flex;
            justify-content: center;
            align-items: flex-start;
            overflow: auto;
            max-height: 800px;
        }}
        .image-container img {{
            max-width: 100%;
            height: auto;
            border: 1px solid #ddd;
            background: white;
        }}
        .text-container {{
            padding: 15px;
            background: #fafafa;
            overflow: auto;
            max-height: 800px;
        }}
        .ocr-text {{
            font-family: "Courier New", Courier, monospace;
            font-size: 12px;
            line-height: 1.4;
            white-space: pre;
            background: white;
            padding: 15px;
            border: 1px solid #ddd;
            border-radius: 2px;
            color: #222;
        }}
        {}
        @media (max-width: 1200px) {{
            .side-by-side {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <h1 style="margin-bottom: 20px; color: #333;">IBM 1130 OCR Comparison View</h1>
"#,
        grid_css
    )
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
        Commands::Analyze {
            scan_set,
            use_llm,
            use_vision,
            vision_model,
        } => {
            analyze_scan_set(&scan_set, use_llm, use_vision, &vision_model).await?;
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
        Commands::TextDump { scan_set, output } => {
            text_dump_scan_set(&scan_set, &output)?;
            Ok(())
        }
        Commands::Compare {
            scan_set,
            output,
            show_grid,
        } => {
            generate_comparison_html(&scan_set, &output, show_grid)?;
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
