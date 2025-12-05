pub use trait_schema_impl::trait_schema;
pub use trait_schema_types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    #[test]
    fn types_reexported() {
        // Ensure the re-exported types are accessible and constructible
        let schema = TraitSchema {
            name: "MyTrait".to_string(),
            functions: vec![],
            cffi_generic_specialization: None,
        };
        assert_eq!(schema.name, "MyTrait");
        assert!(schema.functions.is_empty());
    }

    #[test]
    fn trait_schema_into_tokenstream_contains_name() {
        // Ensure the Into<TokenStream> impl (from trait_schema_types) works
        let schema = TraitSchema {
            name: "MyTrait".to_string(),
            functions: vec![],
            cffi_generic_specialization: None,
        };
        let tokens: TokenStream = schema.into();
        let s = tokens.to_string();
        // The generated token stream should include the trait name string
        assert!(s.contains("MyTrait"));
    }
}
