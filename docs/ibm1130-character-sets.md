# IBM 1130 Character Sets Research

## Problem Statement

Scanned documents may have been printed by different devices with different character sets:

1. **IBM 1132 Printer** - Various models with different type faces
2. **IBM 1403 Printer** - Various models with different print chains
3. **IBM Selectric Console Terminal** - Various type balls

Each device may have had:
- Standard/default character sets
- Optional/custom character sets
- Different models with different capabilities

## Known Character Set Constraints

From initial research (docs/research.txt:462):
- Punch cards use: `[A-Z 0-9 + - * / , . ( ) & # ]`
- This is a subset of characters, not the full IBM 1130 character set

## Character Set Options by Device

### IBM 1403 Printer
- Line printer with **interchangeable print chains**
- Print chains determined available characters
- Common chains:
  - **A-chain** (Standard): Upper case letters, numbers, basic punctuation
  - **H-chain**: Extended special characters
  - **Custom chains**: Customer-specific character sets
- Most common: 48-character or 64-character sets

### IBM 1132 Printer
- Smaller line printer for 1130 systems
- Also used interchangeable print wheels/chains
- Likely similar character set options to 1403
- Documentation: Need to verify standard character set

### IBM Selectric Console Terminal
- Used interchangeable **type balls** (golf ball typing element)
- Different type balls = different character sets
- Most common: Standard PICA, ELITE character sets
- Special scientific/mathematical type balls available

## Proposed Solution for Phase 1 (Non-LLM Baseline)

Since we cannot determine which printer/character set was used for any given scan:

### Strategy 1: Permissive Union Set (Recommended for Phase 1)
Accept the **union of all common IBM 1130-era characters**:

```
ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789
+-*/=<>()[]{}.,;:'"!?&|#@$%
Space and common whitespace
```

### Strategy 2: Configurable Character Sets
Implement multiple character set profiles:

```rust
pub enum CharacterSet {
    /// Punch card standard (limited set)
    PunchCard,
    /// IBM 1403 A-chain (48 chars)
    Ibm1403A,
    /// IBM 1403 H-chain (64 chars)
    Ibm1403H,
    /// IBM 1132 standard
    Ibm1132,
    /// Selectric standard
    Selectric,
    /// Union of all (permissive)
    All,
}
```

### Implementation Plan

**For Phase 1 (TDD approach)**:
1. Start with **permissive/union** character set
2. Use Tesseract with minimal whitelist constraints
3. Let Tesseract's trained models handle character recognition
4. Focus on **layout preservation** (80-column cards, fixed-width listings)

**For Phase 2 (LLM integration)**:
1. LLM can help identify likely printer based on visual cues
2. LLM can suggest character corrections based on context
3. LLM can detect and flag unusual/impossible characters

## Tesseract Configuration Approach

### Option A: No character whitelist (Recommended for initial implementation)
- Let Tesseract use its full trained model
- Filter/validate characters in post-processing
- Easier to implement and test

### Option B: Character whitelist
```rust
tesseract.set_variable("tesseract_char_whitelist",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-*/=()., ")
```
- More restrictive
- May miss valid characters
- Harder to configure correctly without knowing exact printer

## Recommendation

**Start with Option A** (no whitelist) for the following reasons:

1. **We don't know the printer** - Cannot confidently restrict character set
2. **Tesseract is well-trained** - Modern Tesseract handles uppercase well
3. **Post-processing is easier** - Can validate/filter after OCR
4. **TDD-friendly** - Simpler to test without complex configuration
5. **Phase 2 LLM** - Will handle refinement and corrections

## Next Steps

1. Implement basic Tesseract integration without character whitelist
2. Add character set validation as post-processing step
3. Document expected character sets in CIR metadata
4. Allow LLM (Phase 2) to suggest corrections
5. Add manual review UI for uncertain characters

## References

- docs/research.txt - Initial character set constraints
- docs/starting-prompt.txt - Project requirements
- IBM 1130 documentation needed:
  - [ ] IBM 1403 Print Chain specifications
  - [ ] IBM 1132 Printer manual
  - [ ] IBM 1130 System Reference Manual
