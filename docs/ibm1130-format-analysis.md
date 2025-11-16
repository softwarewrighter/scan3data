# IBM 1130 Listing Format Analysis

## Overview

IBM 1130 listings follow strict columnar formats based on 80-column punch cards. Understanding these conventions is critical for:
1. Inferring indentation from partial OCR
2. Creating accurate vision model prompts
3. Reconstructing proper layout from corrupted text

## Standard IBM 1130 Listing Formats

### Format 1: Assembler Object Listing

**Column Layout** (80 columns total):
```
Columns  1-4  : Location (hex address, 4 chars)
Column   5    : Flag (blank, =, -, ', etc.)
Columns  6-9  : Object code word 1 (hex, 4 chars)
Columns 10-13 : Object code word 2 (hex, 4 chars, optional)
Columns 14-19 : DC/BSS type + operand (varies)
Columns 20-35 : Additional data or continuation
Columns 36-80 : Comment/label/instruction (varies by line type)
```

**Example from OCR output**:
```
OBFO    0       0078    DC      123             *NUMBER
|||+- location
||+--- flag (blank/dash/equals)
|+---- word 1
+----- word 2
       DC = Define Constant opcode
              123 = decimal value
                                 *NUMBER = comment
```

### Format 2: Assembler Source Listing

**Column Layout**:
```
Columns  1-5  : Line number or label
Columns  6-8  : Blank
Columns  9-12 : Operation (LDX, STX, BSI, etc.)
Columns 13-14 : Blank
Columns 15-27 : Operand
Columns 28-80 : Comments (often starting with * or ;)
```

### Format 3: Forth Source (Chuck Moore Style)

**Column Layout** (variable indentation):
```
Columns  1-?  : Indentation (0-20+ spaces)
Columns  ?-?  : Forth word definition
Columns  ?-80 : Stack comments, implementation
```

**Indentation Rules**:
- Level 0: Top-level word definitions (`: WORD`)
- Level 1: Primary code body (4 spaces)
- Level 2: Conditional/loop blocks (8 spaces)
- Level 3: Nested structures (12 spaces)

Example:
```
: MYWORD
    DUP 0< IF       ( check if negative )
        NEGATE      ( make positive )
    THEN
    ;
```

### Format 4: FORTRAN Listing

**Column Layout** (classic FORTRAN 66/77):
```
Column   1    : Statement label position (or blank)
Columns  2-5  : Statement label (numeric)
Column   6    : Continuation indicator (blank = new, 1-9 = cont)
Columns  7-72 : Statement text
Columns 73-80 : Sequence number (ignored by compiler)
```

## Indentation Inference Strategies

### Strategy 1: Line Type Recognition

**Identify line types by content patterns**:

1. **Location Lines** (assembler object code):
   - Pattern: `^[0-9A-F]{4}[- =']`
   - Indentation: 0 spaces (column 1)
   - Example: `OBFO 0 0078 DC 123`

2. **Label Lines** (assembler source):
   - Pattern: `^[A-Z][A-Z0-9]{0,4}[ \t]`
   - Indentation: 0 spaces (column 1)
   - Example: `START LDX  L 1`

3. **Instruction Lines** (assembler source):
   - Pattern: `^[ \t]+[A-Z]{3}[ \t]`
   - Indentation: 6-8 spaces (after label field)
   - Example: `      BSI  L SUBR`

4. **Comment Lines**:
   - Pattern: `^[ \t]*[*;]`
   - Indentation: varies (often 0 or aligned with code)
   - Example: `* THIS IS A COMMENT`

5. **Forth Definition Start**:
   - Pattern: `^:[ \t]+[A-Z]`
   - Indentation: 0 spaces
   - Example: `: MYWORD`

6. **Forth Code Body**:
   - Pattern: `^[ \t]+[A-Z]` (but not `:`)
   - Indentation: 4+ spaces (based on nesting)
   - Example: `    DUP 0<`

7. **Forth Semicolon End**:
   - Pattern: `^[ \t]+;`
   - Indentation: 4 spaces (primary level)
   - Example: `    ;`

### Strategy 2: Column Position Inference

**Use later columns to determine indentation**:

1. **DC/BSS Instructions** (Define Constant/Block Storage):
   - If "DC" appears in columns 14-19 -> Location starts at column 1
   - If "BSS" appears in columns 14-19 -> Location starts at column 1
   - Pattern: `DC|BSS|DEC|DECS|BSC`

2. **Hex Addresses**:
   - 4-digit hex at start -> column 1
   - 4-digit hex after spaces -> determine offset
   - Pattern: `[0-9A-F]{4}`

3. **Opcode Recognition**:
   - Three-letter opcodes (LDX, STX, ADD, etc.) -> typically column 9-12
   - If found, backtrack to find label field start (column 1)

4. **Sequence Numbers**:
   - If columns 73-80 contain digits -> FORTRAN format
   - Actual code is columns 7-72
   - Indentation is relative to column 7

### Strategy 3: Consistency Detection

**Multi-line pattern analysis**:

1. **Vertical Alignment**:
   - Collect all lines with same pattern
   - Find modal (most common) column positions
   - Example: All "DC" instructions align at same column

2. **Relative Indentation**:
   - Measure indent relative to previous line
   - IF/THEN/ELSE blocks nest consistently
   - Example: After "IF" -> increase indent 4 spaces

3. **Opcode Columns**:
   - All opcodes should align vertically
   - Determine their column position
   - Infer label field and indentation from there

## Vision Model Prompt Strategy

### Context to Provide to Vision Model

```
You are analyzing an IBM 1130 assembler/Forth listing scan.

COLUMN LAYOUT RULES:
1. Assembler Object Code:
   - Columns 1-4: Location (hex address)
   - Column 5: Flag (-, =, ', or blank)
   - Columns 6-9: Object word 1
   - Columns 10-13: Object word 2 (optional)
   - Columns 14-19: Opcode (DC, BSS, etc.)
   - Columns 20+: Operands and comments

2. Assembler Source:
   - Columns 1-5: Label (left-aligned)
   - Columns 9-12: Opcode (e.g., LDX, STX, BSI)
   - Columns 15+: Operands
   - Columns 40+: Comments (after *)

3. Forth Code:
   - Column 1: Top-level definitions (: WORD)
   - 4 spaces: Primary code
   - 8 spaces: Nested blocks (IF, DO, etc.)
   - 12+ spaces: Deeply nested

INDENTATION INFERENCE:
- Lines starting with hex address (4 digits): column 1
- Lines with "DC", "BSS": column 1 address field
- Lines with 3-letter opcodes: column 9-12 (label at column 1)
- Comment lines (*): align with surrounding code
- Blank lines: preserve exactly

TASK:
Correct the OCR text while preserving EXACT column positions.
Use the visual spacing you see in the image, not the corrupted OCR spacing.
```

### Two-Pass Approach

**Pass 1: Layout Detection**
```
Analyze the image and determine:
1. What format is this? (assembler obj/src, Forth, FORTRAN)
2. What are the column boundaries for each field?
3. Where do indentation levels occur?

Output JSON:
{
  "format": "assembler_object",
  "columns": {
    "location": [1, 4],
    "flag": 5,
    "word1": [6, 9],
    "opcode": [14, 19],
    "operands": [20, 35],
    "comments": [36, 80]
  },
  "indentation_levels": [0, 4, 8, 12]
}
```

**Pass 2: OCR Correction with Layout**
```
Given the layout from Pass 1 and the raw OCR text:
1. Reconstruct each line with correct column positions
2. Fix character errors (Be -> DC, oc -> OC)
3. Preserve exact spacing based on visual analysis

Output:
- Corrected text with proper indentation
- Confidence score per line
- List of corrections made
```

## Reference Column Positions from Samples

Analyzing current OCR output patterns:

```
OBFO-0-0078 BE 123---NUMBER ;
||| | |||| || |||   ||||||| |
123 5 6789 ... (positions)
```

Expected format:
```
OBF0    0       0078    DC      123             *NUMBER
|||++   |       ||||    ||      |||             |||||||
1234    6       10      14      20              36
+- loc  +- w1   +- w2   +- opc  +- operand     +- comment
```

## IBM 1130 Specific Opcodes

**Storage Directives** (column 14+):
- DC (Define Constant)
- BSS (Block Started by Symbol)
- DEC (Decimal)
- DECS (Decimal String)
- BSC (Binary Synchronous Card)

**Instructions** (column 9-12):
- LDX, STX (Load/Store Index)
- LD, STO (Load/Store Accumulator)
- ADD, SUB, MPY, DIV
- BSI (Branch and Store Instruction - subroutine call)
- MDX (Modify Index and Skip)
- AND, OR, XOR, EOR

**Forth Words** (Chuck Moore's 1130 Forth):
- DUP, SWAP, DROP, OVER
- @, !, +!, C@, C!
- IF, THEN, ELSE, DO, LOOP
- : (colon - start definition)
- ; (semicolon - end definition)

## Next Steps for Implementation

1. **Create line type classifier**:
   - Regex patterns for each line type
   - Confidence scoring
   - Column position extraction

2. **Build indentation inference engine**:
   - Analyze vertical alignment patterns
   - Detect modal column positions
   - Infer missing indentation from context

3. **Vision model integration**:
   - Pass image + OCR + inferred layout
   - Request column-aligned reconstruction
   - Validate output against format rules

4. **Validation rules**:
   - Hex addresses are 4 chars in columns 1-4
   - Opcodes align vertically
   - Comments preserve position
   - Total line length <= 80 columns
