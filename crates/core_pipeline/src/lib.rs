//! Core pipeline for scan2data
//!
//! This crate provides the fundamental data structures and processing
//! logic for converting scanned images of IBM 1130 punch cards and
//! listings into structured data suitable for emulator consumption.

pub mod decoder;
pub mod ocr;
pub mod preprocess;
pub mod types;

pub use types::*;
