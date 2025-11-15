//! Core types for the scan2data pipeline
//!
//! This module defines the Canonical Intermediate Representation (CIR)
//! used throughout the processing pipeline.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a scan set (collection of related scans)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScanSetId(pub Uuid);

impl ScanSetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ScanSetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a page artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PageId(pub Uuid);

impl PageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PageId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a card artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId(pub Uuid);

impl CardId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CardId {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of artifact content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    /// Text source card (assembler, FORTRAN, etc.)
    CardText,
    /// Binary/object deck card
    CardObject,
    /// Data-only card
    CardData,
    /// Source listing (assembler/compiler input)
    ListingSource,
    /// Listing including object code
    ListingObject,
    /// Runtime output/log
    RuntimeOutput,
    /// Unknown or unclassified
    Unknown,
}

/// Metadata for a page artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    /// SHA-256 hash of the image content (for duplicate detection)
    pub content_hash: String,
    /// All original filenames that map to this image (duplicate detection)
    pub original_filenames: Vec<String>,
    /// Detected page number (if present in header/footer)
    pub page_number: Option<u32>,
    /// Detected header text
    pub header: Option<String>,
    /// Detected footer text
    pub footer: Option<String>,
    /// Notes about this page (e.g., "interpolated", "damaged")
    pub notes: Vec<String>,
    /// Confidence score for classification (0.0-1.0)
    pub confidence: f32,
}

impl Default for PageMetadata {
    fn default() -> Self {
        Self {
            content_hash: String::new(),
            original_filenames: Vec::new(),
            page_number: None,
            header: None,
            footer: None,
            notes: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// Metadata for a card artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardMetadata {
    /// SHA-256 hash of the image content (for duplicate detection)
    pub content_hash: String,
    /// All original filenames that map to this image (duplicate detection)
    pub original_filenames: Vec<String>,
    /// Sequence number from columns 73-80 (if detected)
    pub sequence_number: Option<String>,
    /// Deck name (if detected from control cards)
    pub deck_name: Option<String>,
    /// Comment from label area
    pub label_comment: Option<String>,
    /// Notes about this card
    pub notes: Vec<String>,
    /// Confidence score for classification (0.0-1.0)
    pub confidence: f32,
}

impl Default for CardMetadata {
    fn default() -> Self {
        Self {
            content_hash: String::new(),
            original_filenames: Vec::new(),
            sequence_number: None,
            deck_name: None,
            label_comment: None,
            notes: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// A page artifact from a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtifact {
    /// Unique identifier
    pub id: PageId,
    /// Parent scan set
    pub scan_set: ScanSetId,
    /// Path to raw scanned image
    pub raw_image_path: PathBuf,
    /// Path to preprocessed image (if processed)
    pub processed_image_path: Option<PathBuf>,
    /// Classification of this page
    pub layout_label: ArtifactKind,
    /// OCR or LLM-extracted text content
    pub content_text: Option<String>,
    /// Metadata extracted from the page
    pub metadata: PageMetadata,
}

/// A card artifact from a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardArtifact {
    /// Unique identifier
    pub id: CardId,
    /// Parent scan set
    pub scan_set: ScanSetId,
    /// Path to raw scanned image
    pub raw_image_path: PathBuf,
    /// Path to preprocessed image (if processed)
    pub processed_image_path: Option<PathBuf>,
    /// Classification of this card
    pub layout_label: ArtifactKind,
    /// Text representation for text decks (80 columns)
    pub text_80col: Option<String>,
    /// Binary representation for object/binary decks (80 bytes)
    pub binary_80col: Option<Vec<u8>>,
    /// Metadata extracted from the card
    pub metadata: CardMetadata,
}

/// High-level artifact after reconstruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HighLevelArtifact {
    /// Reconstructed source listing
    SourceListing(SourceListing),
    /// Reconstructed object deck
    ObjectDeck(ObjectDeck),
    /// Runtime execution log
    RunListing(RunListing),
    /// Mixed or unresolved artifact
    Mixed(MixedArtifact),
}

/// A reconstructed source listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceListing {
    /// Type of source (assembler, FORTRAN, Forth, etc.)
    pub language: String,
    /// Original page artifacts
    pub pages: Vec<PageId>,
    /// Reconstructed lines
    pub lines: Vec<SourceLine>,
}

/// A single line of source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLine {
    /// Line number (if present in source)
    pub line_no: Option<u32>,
    /// Source text
    pub text: String,
    /// True if this line is inferred/reconstructed vs original
    pub inferred: bool,
}

/// A reconstructed object deck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDeck {
    /// Deck name
    pub name: String,
    /// Original card artifacts
    pub cards: Vec<CardId>,
    /// Parsed object cards
    pub object_cards: Vec<ObjectCard>,
}

/// A parsed object/binary card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCard {
    /// Card type identifier
    pub card_type: ObjectCardType,
    /// Load address (if applicable)
    pub address: Option<u16>,
    /// Binary data
    pub data: Vec<u8>,
    /// Symbol references (if any)
    pub symbols: Vec<String>,
}

/// Types of object deck cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectCardType {
    /// Header card
    Header,
    /// Text/code card
    Text,
    /// Relocation card
    Relocation,
    /// Symbol definition
    SymbolDef,
    /// End card
    End,
    /// Unknown/other
    Other,
}

/// A runtime listing (execution log)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunListing {
    /// Original page artifacts
    pub pages: Vec<PageId>,
    /// Log lines
    pub lines: Vec<String>,
}

/// A mixed or unresolved artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedArtifact {
    /// Pages in this artifact
    pub pages: Vec<PageId>,
    /// Cards in this artifact
    pub cards: Vec<CardId>,
    /// Description of the mixture
    pub description: String,
}

/// Output format for IBM 1130 emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EmulatorOutput {
    /// Card deck format
    #[serde(rename = "card_deck")]
    CardDeck {
        /// Target machine
        machine: String,
        /// Cards in the deck
        cards: Vec<EmulatorCard>,
    },
    /// Disk file format
    #[serde(rename = "listing")]
    Listing {
        /// Source language
        language: String,
        /// Lines in the file
        lines: Vec<EmulatorLine>,
    },
}

/// A card in emulator format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorCard {
    /// Sequence number
    pub seq: u32,
    /// 80-column text
    pub text: String,
}

/// A line in emulator format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorLine {
    /// Line number
    pub line_no: u32,
    /// Line text
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_set_id_creation() {
        let id1 = ScanSetId::new();
        let id2 = ScanSetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_artifact_kind_serialization() {
        let kind = ArtifactKind::CardText;
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: ArtifactKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn test_emulator_output_card_deck() {
        let output = EmulatorOutput::CardDeck {
            machine: "IBM1130".to_string(),
            cards: vec![EmulatorCard {
                seq: 10,
                text: "      X21     0100  START".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("\"type\": \"card_deck\""));
        assert!(json.contains("IBM1130"));
    }
}
