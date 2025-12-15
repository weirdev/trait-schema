# Trait Schema - Copilot Instructions

This repository contains a multi-crate Rust project demonstrating a procedural macro that emits a "schema" representation for traits, useful for generating FFI bindings and trait metadata.

## Architecture

- **`trait_schema/`** — convenience re-export crate; consumers depend on this. Exports the `#[trait_schema]` macro and all types.
- **`trait_schema_impl/`** — proc-macro crate containing the core `#[trait_schema]` attribute macro implementation. Parses `ItemTrait` and generates a `<TraitName>_schema()` function.
- **`trait_schema_types/`** — shared types (`TraitSchema`, `FunctionSchema`, `FunctionArgSchema`, `FnArgAnnotations`, `FunctionAnnotations`, `GenericParamSchema`, `GenericParamAnnotations`). All implement `Into<proc_macro2::TokenStream>` for compile-time serialization.
- **`trait_schema_test/`** — example binary demonstrating various trait annotations and output.

## Build & Test

No single workspace root manifest; run cargo per-crate or use `--manifest-path`. From repo root:
```powershell
# Test the macro implementation
cargo test --manifest-path trait_schema_impl/Cargo.toml

# Run examples
cargo run --manifest-path trait_schema_test/Cargo.toml
```

## Key Patterns

- **Generated function naming**: The macro creates `<TraitIdent>_schema()` using `format_ident!("{}_schema", trait_ident)` and wraps it with `#[allow(non_snake_case)]` since trait names are `CamelCase`.
- **String-based representation**: All types/values stored as `String` (e.g., `"Vec<String>"`, `"i32"`). The macro uses `quote!` to stringify; changing to structured types requires updates to `Into<TokenStream>` impls.
- **Argument annotations** (`#[arg(...)]`):
  - `collection_as_item`: Marks a collection parameter as a single item
  - `assert_len = <usize>`: Expected collection length  
  - `cffi_type = "<type>"`: Custom FFI type representation (e.g., `"ptr<i32>"`, `"opt_ptr<f32>"`)
  
  Parsing in `trait_schema_impl/src/lib.rs::process_fn_arg_annotations()` → `FnArgAnnotations`
- **Function annotations** (`#[func(...)]`):
  - `cffi_impl_no_op`: Marks function as no-op for FFI codegen
  
  Parsing in `trait_schema_impl/src/lib.rs::process_fn_annotations()` → `FunctionAnnotations`
- **Trait-level generic annotations** (e.g., `#[trait_schema(T = "ptr<void>")]`): Parsed into `GenericParamAnnotations` with `cffi_type` field.
- **Self parameter skipping**: The macro uses `.skip(1)` when collecting arguments—intentionally excludes the implicit `&self` parameter.

## Dependencies & Integration

- **`syn` v2 & `quote`** (in `trait_schema_impl/Cargo.toml`): Use v2 API for parsing.
- **`Into<TokenStream>` pattern**: Types in `trait_schema_types` convert to token streams for inline code generation. Update these impls whenever schema fields change.
- **Consumer usage**: Depend on `trait_schema` re-export crate; use `#[trait_schema]` on trait definitions. Generated `<TraitName>_schema()` returns runtime metadata.

## Code Change Workflow

1. **Schema structure changes**: Update `trait_schema_types` struct definitions first, then update `Into<TokenStream>` impls to emit valid Rust code.
2. **New attribute keys**: Add parsing in `trait_schema_impl/src/lib.rs`, add field to the corresponding type struct in `trait_schema_types`, add unit tests using `syn::parse_quote!`.
3. **Testing**: Run `cargo test --manifest-path trait_schema_impl/Cargo.toml`, then compile `trait_schema_test/Cargo.toml` to verify end-to-end.

## Examples

- Argument annotations: `#[arg(collection_as_item, assert_len = 1)]` (see `trait_schema_test/src/main.rs::MyTrait`)
- Generic trait annotation: `#[trait_schema(T = "ptr<void>")]` on `trait SpecializedTrait<T>`  
- Function annotation: `#[func(cffi_impl_no_op)]` on a method
