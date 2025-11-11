This repository contains a small multi-crate Rust project that demonstrates a procedural macro which emits a "schema" representation for traits.

Key points an AI agent should know to be immediately productive:

- Architecture (high-level)
  - `trait_schema/` — a convenience crate that re-exports the proc-macro and shared types. See `trait_schema/src/lib.rs`.
  - `trait_schema_impl/` — proc-macro crate. The attribute macro is `#[trait_schema]` implemented in `trait_schema_impl/src/lib.rs`. It parses an `ItemTrait` and emits a function named `<TraitName>_schema()` that returns a `TraitSchema`.
  - `trait_schema_types/` — shared types used by the macro (`TraitSchema`, `FunctionSchema`, `FunctionArgSchema`, `FnArgAnnotations`). These types implement `Into<proc_macro2::TokenStream>` to allow the macro to serialize them into generated code. See `trait_schema_types/src/lib.rs` for generation and Display examples.
  - `trait_schema_test/` — example binary that uses `#[trait_schema]` on several traits and prints the generated schemas. See `trait_schema_test/src/main.rs` for concrete usage examples and expected output.

- Build / test workflows (how to run things)
  - There is no single workspace Cargo manifest at the repo root. Run cargo per crate or use `--manifest-path` from the repo root.
    - Build or test the proc-macro crate:
      ```powershell
      cd trait_schema_impl
      cargo test
      ```
      or from repo root:
      ```powershell
      cargo test --manifest-path trait_schema_impl/Cargo.toml
      ```
    - Run the example binary:
      ```powershell
      cd trait_schema_test
      cargo run
      ```
      or from repo root:
      ```powershell
      cargo run --manifest-path trait_schema_test/Cargo.toml
      ```
  - The tests in `trait_schema_impl` and `trait_schema_types` are small unit tests that exercise parsing and the token generation helpers. Prefer running those crates' tests when making changes to parsing/generation.

- Important repository-specific patterns & conventions
  - The proc-macro emits an extra function named `<TraitIdent>_schema` (note the macro uses `format_ident!("{}_schema", trait_ident)`). The generated function intentionally uses `#[allow(non_snake_case)]` because trait names are `CamelCase`.
  - Types and values in schemas are represented as `String` values (e.g., argument types are stored as strings like `"Vec<String>"`). The macro uses `quote!` to stringify types and names — be careful when changing representation.
  - Argument annotation parsing: function args can have `#[arg(...)]` attributes with keys currently supported:
    - `collection_as_item` (boolean)
    - `assert_len = <usize>`
    See parsing logic in `trait_schema_impl/src/lib.rs::process_fn_arg_annotations` and the annotations struct in `trait_schema_types/src/lib.rs::FnArgAnnotations`.
  - The macro currently skips the first input by `.skip(1)` when building argument lists (it intentionally skips the `self` parameter in trait methods). This is a deliberate implementation detail to be aware of.

- Integration points & dependencies
  - `trait_schema_impl` depends on `syn` and `quote` (see `trait_schema_impl/Cargo.toml`). Changes to parsing should be made with `syn` v2 API usage in mind.
  - `trait_schema_types` provides `Into<proc_macro2::TokenStream>` impls so the macro can directly convert runtime structs into compile-time token streams.
  - Consumers should depend on `trait_schema` (the re-export crate) to get the proc-macro and types: `pub use trait_schema_impl::trait_schema; pub use trait_schema_types::*;`.

- Examples to cite quickly
  - `trait_schema_impl/src/lib.rs` — shows how `ItemTrait` is parsed, how function signatures are inspected, and how `#[arg(...)]` attributes are handled.
  - `trait_schema_types/src/lib.rs` — shows the concrete shape of `TraitSchema`, `FunctionSchema`, `FunctionArgSchema`, and `FnArgAnnotations`, plus `Into<TokenStream>` implementations used by the macro.
  - `trait_schema_test/src/main.rs` — multiple annotated trait examples you can copy to test new parsing or generation behavior.

- Quick guidance for code changes
  - If you change how a schema is represented (e.g., change a field type from String -> structured type), update `trait_schema_types` first and then update `trait_schema_impl`'s code generation so `Into<TokenStream>` still produces valid Rust code at compile-time.
  - When modifying attribute parsing, add focused unit tests in `trait_schema_impl` using `syn::parse_quote!` (the repo already follows this pattern).
  - Prefer small, isolated changes; proc-macro changes can produce confusing compile errors in consumer crates — test changes by running `cargo test` in `trait_schema_impl` and then compiling `trait_schema_test`.

If any section is unclear or you'd like me to add CI snippets (GitHub Actions) or examples of common edits (e.g., adding a new annotation key), tell me which part to expand and I'll iterate.
