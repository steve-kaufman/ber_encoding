# AGENTS.md

This file contains information for AI coding agents working on this project.

## Project Overview

This is a Rust library crate that provides utilities to encode and decode various types using BER (Basic Encoding Rules), as defined in ITU-T X.690.

See [implementation-plan.md](implementation-plan.md) for the full implementation plan.

## Commands

### Testing
```bash
make test
```
This runs `simple-rust-cov` which executes tests with coverage analysis.

**Important:** Always use `make test` rather than `cargo test` directly.

### Building
```bash
make test
```
The test target will build the project as part of running tests.

## Project Structure

```
ber_encoding/
├── src/
│   ├── lib.rs              # Re-exports and crate root
│   ├── error.rs            # BerError type
│   ├── tag.rs              # Tag encoding/decoding
│   ├── length.rs           # Length encoding/decoding
│   ├── traits.rs           # BerTag, BerEncode, BerDecode
│   └── types/              # Type implementations
│       ├── mod.rs
│       ├── boolean.rs
│       ├── integer.rs
│       ├── null.rs
│       ├── enumerated.rs
│       ├── octet_string.rs
│       ├── bit_string.rs
│       ├── sequence.rs
│       └── sequence_of.rs
├── ber_encoding_derive/    # Proc macro crate (future)
├── tests/                  # Integration tests
├── examples/               # Example usage
└── Makefile
```

## Coding Conventions

### General
- Follow Rust standard library conventions and idioms
- Use descriptive variable names
- Prefer explicit error handling over panics
- Document all public APIs with doc comments

### Error Handling
- Use `BerError` for all BER-related errors
- Propagate errors with `?` operator
- Provide context in error variants when helpful

### Testing
- Write unit tests in the same file as the implementation using `#[cfg(test)]` modules
- Write integration tests in the `tests/` directory
- Use test vectors from BER/ASN.1 specifications when available
- Test round-trip encoding/decoding: `decode(encode(x)) == x`
- Test edge cases: zero, negative, maximum values, empty collections

### Documentation
- All public items must have doc comments
- Include examples in doc comments where helpful
- Reference ITU-T X.690 sections when implementing specific behaviors

## Key Architecture Decisions

### Three Core Traits
1. **BerTag**: Allows types to declare their BER tag
2. **BerEncode**: Encodes a type to BER format (requires BerTag)
3. **BerDecode**: Decodes a type from BER format (requires BerTag)

### Tag-Length-Value Structure
All BER encodings follow the TLV pattern:
- Tag: identifies the type
- Length: number of octets in value
- Value: actual data

### Derive Macro Pattern
Follow serde-like patterns for struct annotations:
```rust
#[derive(BerEncode, BerDecode, BerTag)]
struct Example {
    #[ber(tag = 0)]
    field: i32,
}
```

## Reference Documentation

- ITU-T X.690: https://www.itu.int/rec/T-REC-X.690/
- BER encoding rules specification
