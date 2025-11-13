# BER Encoding Crate Implementation Plan

## Architecture Overview

The crate will be structured in layers: core BER primitives → traits → type implementations → proc macro for ergonomic struct encoding.

```
Core Layer (Tag, Length, Value) 
    ↓
Trait Layer (BerTag, BerEncode, BerDecode)
    ↓
Type Implementations (Boolean, Integer, etc.)
    ↓
Derive Macro (for structs)
```

## Phase 1: Core BER Primitives ✅ COMPLETED

### Tag Module (`src/tag.rs`)
- Implement `Tag` struct representing BER tag identifiers
- Support tag class (Universal, Application, Context-specific, Private)
- Support constructed vs primitive flag
- Support tag number encoding (short form <31, long form ≥31)
- Methods: `new()`, `is_constructed()`, `tag_number()`, `encode()`, `decode()`

### Length Module (`src/length.rs`)
- Implement `Length` enum for definite and indefinite lengths
- Definite length: short form (0-127) and long form (>127)
- Indefinite length support for constructed types
- Methods: `encode()`, `decode()`

### Error Types (`src/error.rs`)
- Define `BerError` enum for all encoding/decoding failures
- Variants: `IoError`, `InvalidTag`, `InvalidLength`, `UnexpectedEof`, `InvalidValue`, etc.
- Implement `std::error::Error` and conversion from `std::io::Error`

## Phase 2: Core Traits ✅ COMPLETED

### BerTag Trait (`src/traits.rs`)
```rust
pub trait BerTag {
    fn tag() -> Tag;
}
```
- Allows types to declare their BER tag independently
- Used by `BerEncode`/`BerDecode` for tag information
- Can be overridden in derive macro with attributes

### BerEncode Trait (`src/traits.rs`)
```rust
pub trait BerEncode: BerTag {
    fn encode(&self, writer: &mut impl Write) -> Result<(), BerError>;
    
    fn encode_with_tag(&self, tag: Tag, writer: &mut impl Write) -> Result<(), BerError> {
        // Allow encoding with custom tag (for context-specific tagging)
    }
}
```
- Core encoding trait for all BER types
- Automatically writes tag-length-value

### BerDecode Trait (`src/traits.rs`)
```rust
pub trait BerDecode: BerTag + Sized {
    fn decode(reader: &mut impl Read) -> Result<Self, BerError>;
    
    fn decode_with_tag(tag: Tag, reader: &mut impl Read) -> Result<Self, BerError> {
        // Allow decoding with expected tag validation
    }
}
```
- Core decoding trait for all BER types
- Validates tag matches expected type

## Phase 3: Primitive Types ❌ NOT STARTED

### Boolean (`src/types/boolean.rs`)
- Universal class, tag 1, primitive
- Implement `BerTag`, `BerEncode`, `BerDecode` for `bool`
- Encoding: 0x00 for false, non-zero (typically 0xFF) for true
- Length is always 1

### Integer (`src/types/integer.rs`)
- Universal class, tag 2, primitive
- Implement for `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- Implement for `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- Two's complement encoding, minimum octets
- Handle sign extension and leading zero removal

### Null (`src/types/null.rs`)
- Universal class, tag 5, primitive
- Create `Null` unit struct
- Length is always 0, no value content

### Enumerated (`src/types/enumerated.rs`)
- Universal class, tag 10, primitive
- Create `Enumerated` wrapper type
- Similar encoding to Integer but different tag
- Consider wrapper: `Enumerated(i32)`

## Phase 4: String Types ❌ NOT STARTED

### OctetString (`src/types/octet_string.rs`)
- Universal class, tag 4, primitive
- Implement `BerTag`, `BerEncode`, `BerDecode` for `Vec<u8>` and `&[u8]`
- Length is byte count
- Value is raw bytes

### BitString (`src/types/bit_string.rs`)
- Universal class, tag 3, primitive
- Create `BitString` struct with data and unused bits count
- First octet: number of unused bits (0-7)
- Remaining octets: bit data
- Methods: `from_bytes()`, `to_bytes()`, `len_bits()`

## Phase 5: Constructed Types ❌ NOT STARTED

### Sequence (`src/types/sequence.rs`)
- Universal class, tag 16, constructed
- Used for ordered collections of different types (like structs)
- Encoding: concatenate encoded fields
- Manual implementation for tuples: `(T1, T2, ...)` where each `Ti: BerEncode`

### SequenceOf (`src/types/sequence_of.rs`)
- Universal class, tag 16, constructed
- Used for ordered collections of same type
- Implement `BerEncode`/`BerDecode` for `Vec<T> where T: BerEncode/BerDecode`
- Create wrapper type to distinguish from OctetString: `SequenceOf<T>(Vec<T>)`

## Phase 6: Derive Macro ❌ NOT STARTED

### Setup
- Create `ber_encoding_derive` crate (proc-macro = true)
- Add dependency: `syn`, `quote`, `proc-macro2`
- Export from main crate: `pub use ber_encoding_derive::*;`

### Derive BerTag
```rust
#[derive(BerTag)]
#[ber(tag_class = "universal", tag_number = 16, constructed = true)]
struct MyStruct { ... }
```
- Generate `BerTag` implementation
- Default to universal class, tag 16, constructed for structs
- Allow override with attributes

### Derive BerEncode
```rust
#[derive(BerEncode, BerTag)]
struct Person {
    #[ber(tag = 0)]
    name: String,
    #[ber(tag = 1)]
    age: i32,
}
```
- Generate implementation that:
  1. Encodes struct tag using `Self::tag()`
  2. Encodes total length of all fields
  3. Encodes each field with context-specific tag (if specified)
- Support field attributes:
  - `#[ber(tag = N)]` - context-specific tag
  - `#[ber(skip)]` - skip field during encoding
  - `#[ber(optional)]` - field is `Option<T>`

### Derive BerDecode
- Generate implementation that:
  1. Reads and validates tag matches `Self::tag()`
  2. Reads length
  3. Decodes each field in order
  4. Handles optional fields
- Match context-specific tags to struct fields

## Phase 7: Testing & Documentation ❌ NOT STARTED

### Unit Tests
- Test each type with known BER-encoded test vectors
- Test edge cases: zero, negative numbers, max values
- Test length encodings: short form, long form boundaries
- Test tag encodings: different classes and tag numbers

### Integration Tests
- Test complex nested structures
- Test derived structs with various configurations
- Test round-trip: encode then decode equals original
- Test interop with other BER libraries (if available)

### Documentation
- API documentation for all public items
- Module-level docs explaining BER concepts
- Usage examples in doc comments
- README.md with:
  - Quick start guide
  - Feature overview
  - Code examples
  - Link to ITU-T X.690 specification

### Examples
- `examples/basic_types.rs` - encoding/decoding primitives
- `examples/sequences.rs` - working with sequences
- `examples/derive_macro.rs` - using the derive macro
- `examples/custom_tags.rs` - context-specific tagging

## Project Structure

```
ber_encoding/
├── src/
│   ├── lib.rs              // Re-exports and crate root
│   ├── error.rs            // BerError type
│   ├── tag.rs              // Tag encoding/decoding
│   ├── length.rs           // Length encoding/decoding
│   ├── traits.rs           // BerTag, BerEncode, BerDecode
│   └── types/
│       ├── mod.rs
│       ├── boolean.rs
│       ├── integer.rs
│       ├── null.rs
│       ├── enumerated.rs
│       ├── octet_string.rs
│       ├── bit_string.rs
│       ├── sequence.rs
│       └── sequence_of.rs
├── ber_encoding_derive/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          // Proc macro implementation
├── tests/
│   ├── integration_tests.rs
│   └── interop_tests.rs
├── examples/
├── Cargo.toml
└── README.md
```

## Dependencies

### Main Crate
- None (std only, or consider `no_std` support later)

### Derive Crate
- `syn = "2.0"`
- `quote = "1.0"`
- `proc-macro2 = "1.0"`

## Future Enhancements (Post v1.0)

- Support for more ASN.1/BER types (UTCTime, GeneralizedTime, etc.)
- DER (Distinguished Encoding Rules) support with stricter encoding
- No-std support with `alloc`
- Async encoding/decoding with `AsyncRead`/`AsyncWrite`
- Better error messages with context
- Schema validation
- Codec for common protocols (SNMP, LDAP, etc.)
