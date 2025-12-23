use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, GenericParam, ItemTrait, LitInt, Receiver, ReturnType, TraitItem,
};

use trait_schema_types as trait_schema;

#[proc_macro_attribute]
/// Attribute macro that captures trait metadata and emits a schema function.
pub fn trait_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemTrait);
    let trait_ident = input.ident.clone();
    // Generated schema function name
    let schema_fn_ident = format_ident!("{}_schema", trait_ident);

    // Parse generic parameter annotations from trait-level attribute
    // Format: #[trait_schema] or #[trait_schema(T = "cffi_type", U = "other_cffi_type")]
    let generic_annotations = parse_generic_annotations_attr(proc_macro2::TokenStream::from(attr));

    // Build generics and functions using helper routines
    let trait_generics = build_trait_generics(&input.generics, &generic_annotations);

    let trait_functions = extract_trait_functions(&mut input.items);

    let trait_supertraits = extract_supertraits(&input.supertraits);

    let trait_name_string = trait_ident.to_string();
    let trait_schema = trait_schema::TraitSchema {
        name: trait_name_string,
        functions: trait_functions,
        generics: trait_generics,
        supertraits: trait_supertraits,
    };

    let trait_tokens: proc_macro2::TokenStream = trait_schema.into();

    let output = quote! {
        #input

        #[allow(non_snake_case)]
        pub fn #schema_fn_ident() -> trait_schema::TraitSchema {
            #trait_tokens
        }
    };

    // Debug formatting output:
    // eprintln!("{:#?}", output);
    // Raw text output:
    // println!("{}", output);

    output.into()
}

/// Parse argument-level annotations (#[arg(...)]) and strip the attribute from the input.
fn process_fn_arg_annotations(arg: &mut FnArg) -> trait_schema::FnArgAnnotations {
    let mut info = trait_schema::FnArgAnnotations::new();
    if let syn::FnArg::Typed(pat_type) = arg {
        for attr in &pat_type.attrs {
            if attr.path().is_ident("arg") {
                // Parse meta inside #[arg(...)]
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("collection_as_item") {
                        info.collection_as_item = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("assert_len") {
                        let v: LitInt = meta.value()?.parse()?;
                        info.assert_len = Some(v.base10_parse::<usize>()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("cffi_type") {
                        let v: syn::LitStr = meta.value()?.parse()?;
                        info.cffi_type = Some(v.value());
                        return Ok(());
                    }
                    // Unknown key -> ignore or error:
                    // return Err(meta.error("unknown arg attribute"));
                    Ok(())
                });
            }
        }
        pat_type.attrs.retain(|a| !a.path().is_ident("arg"));
    }

    info
}

/// Parse function-level annotations on trait methods (#[func(...)]).
fn process_fn_annotations(func: &mut syn::TraitItemFn) -> trait_schema::FunctionAnnotations {
    let mut info = trait_schema::FunctionAnnotations::new();
    for attr in &func.attrs {
        if attr.path().is_ident("func") {
            // Parse nested meta inside #[func(...)]
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("cffi_impl_no_op") {
                    info.cffi_impl_no_op = true;
                    return Ok(());
                }
                Ok(())
            });
        }
    }
    // remove the parsed attribute so downstream consumers don't see it
    func.attrs.retain(|a| !a.path().is_ident("func"));
    info
}

/// Parse generic parameter annotations from trait attribute
/// Format: #[trait_schema] or #[trait_schema(T = "ptr<void>", U = "ptr<f32>")]
fn parse_generic_annotations_attr(
    tokens: proc_macro2::TokenStream,
) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    let mut iter = tokens.into_iter().peekable();
    while iter.peek().is_some() {
        // Expect: identifier
        let param_name = match iter.next() {
            Some(proc_macro2::TokenTree::Ident(ident)) => ident.to_string(),
            _ => break,
        };
        // Expect: =
        match iter.next() {
            Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '=' => {}
            _ => break,
        }
        // Expect: string literal
        let cffi_type = match iter.next() {
            Some(proc_macro2::TokenTree::Literal(lit)) => {
                let lit_str = lit.to_string();
                if lit_str.starts_with('"') && lit_str.ends_with('"') {
                    lit_str[1..lit_str.len() - 1].to_string()
                } else {
                    break;
                }
            }
            _ => break,
        };
        result.insert(param_name, cffi_type);
        // Expect optional: comma
        match iter.peek() {
            Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' => {
                iter.next(); // consume comma
            }
            None => break,
            _ => break,
        }
    }
    result
}

/// Build a vector of `GenericParamSchema` from `syn::Generics` and parsed annotations.
fn build_trait_generics(
    generics: &syn::Generics,
    generic_annotations: &std::collections::HashMap<String, String>,
) -> Vec<trait_schema::GenericParamSchema> {
    let mut trait_generics: Vec<trait_schema::GenericParamSchema> = Vec::new();

    for generic_param in &generics.params {
        if let GenericParam::Type(type_param) = generic_param {
            let param_name = type_param.ident.to_string();
            let cffi_type = generic_annotations.get(&param_name).cloned();
            let annotations = if cffi_type.is_some() {
                Some(trait_schema::GenericParamAnnotations { cffi_type })
            } else {
                None
            };

            trait_generics.push(trait_schema::GenericParamSchema {
                name: param_name,
                annotations,
            });
        }
    }

    trait_generics
}

/// Extract function schemas from the trait items, parsing argument annotations.
fn extract_trait_functions(items: &mut Vec<TraitItem>) -> Vec<trait_schema::FunctionSchema> {
    let mut trait_functions: Vec<trait_schema::FunctionSchema> = Vec::new();

    for it in items.iter_mut() {
        if let TraitItem::Fn(m) = it {
            let fn_annotations = process_fn_annotations(m);
            let sig = &mut m.sig;

            let args: Vec<trait_schema::FunctionArgSchema> = sig
                .inputs
                .iter_mut()
                .filter_map(|arg| {
                    if let FnArg::Typed(pat_type) = arg {
                        let arg_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            format!("{}", quote! { #pat_ident })
                        } else {
                            "".to_string()
                        };

                        let arg_ty = &*pat_type.ty;
                        let arg_ty = format!("{}", quote! { #arg_ty });

                        let annotations = process_fn_arg_annotations(arg);

                        Some(trait_schema::FunctionArgSchema {
                            name: arg_name,
                            ty: Some(arg_ty),
                            annotations: Some(annotations),
                        })
                    } else if let FnArg::Receiver(Receiver { .. }) = arg {
                        Some(trait_schema::FunctionArgSchema {
                            name: "self".to_string(),
                            ty: None,
                            annotations: None,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            let return_type = match &sig.output {
                ReturnType::Default => "()".to_string(),
                ReturnType::Type(_, ty) => format!("{}", quote! { #ty }),
            };

            trait_functions.push(trait_schema::FunctionSchema {
                name: sig.ident.to_string(),
                args,
                return_type,
                body: None,
                extern_layout: None,
                annotations: Some(fn_annotations),
            });
        }
    }

    trait_functions
}

/// Extract supertraits from the trait bounds
fn extract_supertraits(supertraits: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>) -> Vec<String> {
    supertraits
        .iter()
        .filter_map(|bound| {
            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                Some(format!("{}", quote! { #trait_bound }))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_process_fn_arg_annotations_no_annotations() {
        let mut arg: FnArg = parse_quote!(arg1: String);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert!(!annotations.collection_as_item);
        assert!(annotations.assert_len.is_none());
    }

    #[test]
    fn test_process_fn_arg_annotations_collection_as_item() {
        let mut arg: FnArg = parse_quote!(#[arg(collection_as_item)] arg1: Vec<String>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert!(annotations.collection_as_item);
        assert!(annotations.assert_len.is_none());
    }

    #[test]
    fn test_process_fn_arg_annotations_assert_len() {
        let mut arg: FnArg = parse_quote!(#[arg(assert_len = 5)] arg1: Vec<String>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert!(!annotations.collection_as_item);
        assert_eq!(annotations.assert_len, Some(5));
    }

    #[test]
    fn test_process_fn_arg_annotations_combined() {
        let mut arg: FnArg =
            parse_quote!(#[arg(collection_as_item, assert_len = 10)] arg1: Vec<String>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert!(annotations.collection_as_item);
        assert_eq!(annotations.assert_len, Some(10));
    }

    #[test]
    fn test_process_fn_arg_annotations_multiple_values() {
        let mut arg: FnArg = parse_quote!(#[arg(assert_len = 42)] arg1: Vec<i32>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert_eq!(annotations.assert_len, Some(42));
    }

    #[test]
    fn test_process_fn_arg_annotations_removes_arg_attribute() {
        let mut arg: FnArg = parse_quote!(#[arg(collection_as_item)] arg1: Vec<String>);
        process_fn_arg_annotations(&mut arg);

        if let FnArg::Typed(pat_type) = &arg {
            // The #[arg(...)] attribute should be removed
            let has_arg_attr = pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("arg"));
            assert!(!has_arg_attr);
        } else {
            panic!("Expected FnArg::Typed");
        }
    }

    #[test]
    fn test_process_fn_arg_annotations_zero_assert_len() {
        let mut arg: FnArg = parse_quote!(#[arg(assert_len = 0)] arg1: Vec<String>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert_eq!(annotations.assert_len, Some(0));
    }

    #[test]
    fn test_process_fn_arg_annotations_large_assert_len() {
        let mut arg: FnArg = parse_quote!(#[arg(assert_len = 999999)] arg1: Vec<String>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert_eq!(annotations.assert_len, Some(999999));
    }

    #[test]
    fn test_fn_arg_annotations_new() {
        let annotations = trait_schema::FnArgAnnotations::new();
        assert!(!annotations.collection_as_item);
        assert!(annotations.assert_len.is_none());
    }

    #[test]
    fn test_fn_arg_annotations_creation_manual() {
        let annotations = trait_schema::FnArgAnnotations {
            collection_as_item: true,
            assert_len: Some(25),
            cffi_type: Some("opt_ptr<f32>".to_string()),
        };

        assert!(annotations.collection_as_item);
        assert_eq!(annotations.assert_len, Some(25));
        assert_eq!(annotations.cffi_type, Some("opt_ptr<f32>".to_string()));
    }

    #[test]
    fn test_process_fn_arg_annotations_cffi_type() {
        let mut arg: FnArg = parse_quote!(#[arg(cffi_type = "ptr<f64>")] values: Arc<f64>);
        let annotations = process_fn_arg_annotations(&mut arg);

        assert_eq!(annotations.cffi_type, Some("ptr<f64>".to_string()));
        assert!(!annotations.collection_as_item);
        assert!(annotations.assert_len.is_none());

        // Ensure the attribute was removed from the typed arg
        if let FnArg::Typed(pat_type) = &arg {
            let has_arg_attr = pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("arg"));
            assert!(!has_arg_attr);
        } else {
            panic!("Expected FnArg::Typed");
        }
    }

    #[test]
    fn test_parse_generic_annotations_attr_empty() {
        let tokens = proc_macro2::TokenStream::new();
        let result = parse_generic_annotations_attr(tokens);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_generic_annotations_attr_single() {
        let tokens = quote::quote!(T = "ptr<void>");
        let result = parse_generic_annotations_attr(tokens);
        assert_eq!(result.get("T"), Some(&"ptr<void>".to_string()));
    }

    #[test]
    fn test_parse_generic_annotations_attr_multiple() {
        let tokens = quote::quote!(T = "ptr<void>", U = "ptr<f32>");
        let result = parse_generic_annotations_attr(tokens);
        assert_eq!(result.get("T"), Some(&"ptr<void>".to_string()));
        assert_eq!(result.get("U"), Some(&"ptr<f32>".to_string()));
    }

    #[test]
    fn test_build_trait_generics_basic() {
        let generics: syn::Generics = parse_quote!(<T, U>);
        let mut annotations = std::collections::HashMap::new();
        annotations.insert("T".to_string(), "ptr<void>".to_string());
        let result = build_trait_generics(&generics, &annotations);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "T");
        assert_eq!(
            result[0].annotations.as_ref().unwrap().cffi_type,
            Some("ptr<void>".to_string())
        );
        assert_eq!(result[1].name, "U");
        assert!(result[1].annotations.is_none());
    }

    #[test]
    fn test_extract_trait_functions_basic() {
        let mut items: Vec<TraitItem> = vec![
            parse_quote! {
                fn foo(&self, x: i32) -> i32;
            },
            parse_quote! {
                fn bar(&mut self);
            },
        ];
        let result = extract_trait_functions(&mut items);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "foo");
        assert_eq!(result[0].args[0].name, "self");
        assert_eq!(result[0].args[1].name, "x");
        assert_eq!(result[0].return_type, "i32");
        assert_eq!(result[1].name, "bar");
        assert_eq!(result[1].args[0].name, "self");
        assert_eq!(result[1].return_type, "()".to_string());
    }

    #[test]
    fn test_process_fn_annotations_no_attr() {
        let mut func: syn::TraitItemFn = parse_quote!(fn foo(&self););
        let annotations = process_fn_annotations(&mut func);

        assert!(!annotations.cffi_impl_no_op);
        // ensure attribute wasn't present (and thus none removed)
        let has_func_attr = func.attrs.iter().any(|a| a.path().is_ident("func"));
        assert!(!has_func_attr);
    }

    #[test]
    fn test_process_fn_annotations_cffi_impl_no_op() {
        let mut func: syn::TraitItemFn = parse_quote!(#[func(cffi_impl_no_op)] fn foo(&self););
        let annotations = process_fn_annotations(&mut func);

        assert!(annotations.cffi_impl_no_op);

        // The #[func(...)] attribute should be removed from the function
        let has_func_attr = func.attrs.iter().any(|a| a.path().is_ident("func"));
        assert!(!has_func_attr);
    }

    #[test]
    fn test_extract_supertraits_empty() {
        let trait_def: syn::ItemTrait = parse_quote!(trait MyTrait {});
        let result = extract_supertraits(&trait_def.supertraits);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_supertraits_single() {
        let trait_def: syn::ItemTrait = parse_quote!(trait MyTrait: Clone {});
        let result = extract_supertraits(&trait_def.supertraits);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Clone");
    }

    #[test]
    fn test_extract_supertraits_multiple() {
        let trait_def: syn::ItemTrait =
            parse_quote!(trait MyTrait: Clone + Debug + Send {});
        let result = extract_supertraits(&trait_def.supertraits);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Clone");
        assert_eq!(result[1], "Debug");
        assert_eq!(result[2], "Send");
    }

    #[test]
    fn test_extract_supertraits_with_path() {
        let trait_def: syn::ItemTrait =
            parse_quote!(trait MyTrait: std::fmt::Debug {});
        let result = extract_supertraits(&trait_def.supertraits);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "std :: fmt :: Debug");
    }

    #[test]
    fn test_extract_supertraits_generic_trait() {
        let trait_def: syn::ItemTrait =
            parse_quote!(trait MyTrait: IntoIterator<Item = String> {});
        let result = extract_supertraits(&trait_def.supertraits);
        assert_eq!(result.len(), 1);
        // The output includes the full trait with generic parameters
        assert!(result[0].contains("IntoIterator"));
        assert!(result[0].contains("Item"));
        assert!(result[0].contains("String"));
    }

    #[test]
    fn test_extract_supertraits_mixed_lifetimes_and_traits() {
        let trait_def: syn::ItemTrait =
            parse_quote!(trait MyTrait: 'static + Clone + Sync {});
        let result = extract_supertraits(&trait_def.supertraits);
        // 'static is a lifetime bound, not a trait, so it should be filtered out
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Clone");
        assert_eq!(result[1], "Sync");
    }

    #[test]
    fn test_extract_supertraits_complex() {
        let trait_def: syn::ItemTrait = parse_quote!(
            trait MyTrait: Send + Sync + std::fmt::Debug + Clone {}
        );
        let result = extract_supertraits(&trait_def.supertraits);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "Send");
        assert_eq!(result[1], "Sync");
        assert_eq!(result[2], "std :: fmt :: Debug");
        assert_eq!(result[3], "Clone");
    }
}
