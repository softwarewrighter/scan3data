//! Decoder module for IBM 1130 object decks
//!
//! Handles parsing of binary/object deck cards including:
//! - Card type identification
//! - Compressed label column decoding
//! - Address field extraction
//! - Binary data extraction

use crate::types::{ObjectCard, ObjectCardType};
use anyhow::Result;

/// Decode an 80-byte object card
pub fn decode_object_card(data: &[u8]) -> Result<ObjectCard> {
    if data.len() != 80 {
        anyhow::bail!("Object card must be exactly 80 bytes");
    }

    // TODO: Implement IBM 1130 object card format
    // - Parse card type indicator
    // - Extract address fields
    // - Decode compressed labels
    // - Extract binary data

    Ok(ObjectCard {
        card_type: ObjectCardType::Other,
        address: None,
        data: data.to_vec(),
        symbols: Vec::new(),
    })
}

/// Disassemble IBM 1130 machine code
pub fn disassemble_1130(_data: &[u8], start_address: u16) -> Result<Vec<String>> {
    // TODO: Implement IBM 1130 disassembler
    // - Decode opcodes
    // - Format operands
    // - Add labels for branch targets

    let mut result = Vec::new();
    result.push(format!("       ORG  {:04X}", start_address));
    result.push("       ; TODO: Implement disassembler".to_string());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_object_card_length_check() {
        let data = vec![0u8; 79];
        let result = decode_object_card(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_object_card_valid() {
        let data = vec![0u8; 80];
        let result = decode_object_card(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_disassemble_basic() {
        let code = vec![0x00, 0x00, 0x01, 0x00];
        let result = disassemble_1130(&code, 0x0100);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
