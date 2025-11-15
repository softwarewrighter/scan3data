//! Image preprocessing module
//!
//! Handles classical computer vision operations for cleaning scanned images:
//! - Grayscale conversion
//! - Contrast adjustment
//! - Adaptive thresholding
//! - Deskewing
//! - Noise removal
//! - Cropping
//! - Duplicate detection via SHA-256 hashing

use anyhow::Result;
use image::{DynamicImage, GrayImage, ImageBuffer, Rgb};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

/// Preprocess a scanned image for OCR/analysis
pub fn preprocess_image(input: &DynamicImage) -> Result<GrayImage> {
    // Convert to grayscale
    let gray = input.to_luma8();

    // TODO: Add contrast stretching
    // TODO: Add adaptive thresholding
    // TODO: Add morphological operations
    // TODO: Add deskewing (Hough transform)

    Ok(gray)
}

/// Detect and crop individual cards from a multi-card scan
pub fn segment_cards(input: &GrayImage) -> Result<Vec<GrayImage>> {
    // TODO: Implement card segmentation
    // - Edge detection
    // - Contour finding
    // - Bounding box extraction

    Ok(vec![input.clone()])
}

/// Deskew an image using Hough transform
pub fn deskew_image(input: &GrayImage) -> Result<GrayImage> {
    // TODO: Implement deskewing
    // - Find dominant lines
    // - Calculate rotation angle
    // - Rotate image

    Ok(input.clone())
}

/// Compute SHA-256 hash of an image for duplicate detection
///
/// Returns a 64-character hexadecimal string representing the SHA-256 hash
/// of the image's raw pixel data.
pub fn compute_image_hash(image: &RgbImage) -> String {
    let mut hasher = Sha256::new();
    hasher.update(image.as_raw());
    format!("{:x}", hasher.finalize())
}

/// Group representing images with identical content
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// SHA-256 hash of the image content
    pub hash: String,
    /// All filenames that map to this image
    pub filenames: Vec<PathBuf>,
}

/// Type alias for image with RGB pixels
pub type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// Detect duplicate images based on SHA-256 hash
///
/// Takes a list of (filename, image) tuples and returns groups of images
/// with identical content. Each group contains the hash and all filenames
/// that map to that content.
pub fn detect_duplicates(images: &[(PathBuf, RgbImage)]) -> Vec<DuplicateGroup> {
    let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Compute hash for each image and group by hash
    for (filename, image) in images {
        let hash = compute_image_hash(image);
        hash_map.entry(hash).or_default().push(filename.clone());
    }

    // Convert to DuplicateGroup vec
    hash_map
        .into_iter()
        .map(|(hash, filenames)| DuplicateGroup { hash, filenames })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn test_preprocess_basic() {
        let img = ImageBuffer::from_pixel(100, 100, Rgb([255u8, 255u8, 255u8]));
        let dynamic = DynamicImage::ImageRgb8(img);

        let result = preprocess_image(&dynamic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_image_hash_deterministic() {
        // Same image should produce same hash
        let img1 = ImageBuffer::from_pixel(10, 10, Rgb([128u8, 128u8, 128u8]));
        let img2 = ImageBuffer::from_pixel(10, 10, Rgb([128u8, 128u8, 128u8]));

        let hash1 = compute_image_hash(&img1);
        let hash2 = compute_image_hash(&img2);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_compute_image_hash_different_for_different_images() {
        // Different images should produce different hashes
        let img1 = ImageBuffer::from_pixel(10, 10, Rgb([128u8, 128u8, 128u8]));
        let img2 = ImageBuffer::from_pixel(10, 10, Rgb([64u8, 64u8, 64u8]));

        let hash1 = compute_image_hash(&img1);
        let hash2 = compute_image_hash(&img2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_detect_duplicates_finds_identical_images() {
        use std::path::PathBuf;

        let img1 = ImageBuffer::from_pixel(5, 5, Rgb([100u8, 100u8, 100u8]));
        let img2 = ImageBuffer::from_pixel(5, 5, Rgb([100u8, 100u8, 100u8]));
        let img3 = ImageBuffer::from_pixel(5, 5, Rgb([200u8, 200u8, 200u8]));

        let images = vec![
            (PathBuf::from("image1.jpg"), img1),
            (PathBuf::from("image2.jpg"), img2),
            (PathBuf::from("image3.jpg"), img3),
        ];

        let groups = detect_duplicates(&images);

        // Should have 2 groups: one with img1+img2, one with img3
        assert_eq!(groups.len(), 2);

        // Find the duplicate group
        let duplicate_group = groups
            .iter()
            .find(|g| g.filenames.len() == 2)
            .expect("Should find group with 2 duplicates");

        assert_eq!(duplicate_group.filenames.len(), 2);
        assert!(duplicate_group
            .filenames
            .contains(&PathBuf::from("image1.jpg")));
        assert!(duplicate_group
            .filenames
            .contains(&PathBuf::from("image2.jpg")));
    }

    #[test]
    fn test_detect_duplicates_no_duplicates() {
        use std::path::PathBuf;

        let img1 = ImageBuffer::from_pixel(5, 5, Rgb([100u8, 100u8, 100u8]));
        let img2 = ImageBuffer::from_pixel(5, 5, Rgb([150u8, 150u8, 150u8]));
        let img3 = ImageBuffer::from_pixel(5, 5, Rgb([200u8, 200u8, 200u8]));

        let images = vec![
            (PathBuf::from("image1.jpg"), img1),
            (PathBuf::from("image2.jpg"), img2),
            (PathBuf::from("image3.jpg"), img3),
        ];

        let groups = detect_duplicates(&images);

        // Should have 3 groups, each with 1 image
        assert_eq!(groups.len(), 3);
        assert!(groups.iter().all(|g| g.filenames.len() == 1));
    }
}
