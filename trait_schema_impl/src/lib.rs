use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, LitInt, Receiver, ReturnType, TraitItem, parse_macro_input};

use trait_schema_types as trait_schema;

#[proc_macro_attribute]
pub fn trait_schema(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemTrait);
    let trait_ident = input.ident.clone();
    // Generated schema function name
    let schema_fn_ident = format_ident!("{}_schema", trait_ident);

    let mut trait_functions: Vec<trait_schema::FunctionSchema> = Vec::new();

    for it in &mut input.items {
        if let TraitItem::Fn(m) = it {
            let sig = &mut m.sig;

            trait_functions.push(trait_schema::FunctionSchema {
                name: sig.ident.to_string(),
                args: sig
                    .inputs
                    .iter_mut()
                    // TODO: Skipping the self argument for now
                    .skip(1)
                    .filter_map(|arg| {
                        if let FnArg::Typed(pat_type) = arg {
                            let arg_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                format!("{}", quote! { #pat_ident })
                            } else {
                                "".to_string()
                            };
                            // eprintln!("{:#?}", quote! { #pat_type });
                            let arg_ty = &*pat_type.ty;
                            let arg_ty = format!("{}", quote! { #arg_ty });
                            // let ty = match ty {
                            //     Type::Reference(ty_ref) => Some(format!("{}", quote! { #ty_ref }))
                            //     Type::Path(ty_path) => Some(format!("{}", quote! { #ty_path })),
                            //     _ => Some(format!("{}", quote! { #ty })),
                            // };

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
                    .collect(),
                return_type: match &sig.output {
                    ReturnType::Default => "()".to_string(),
                    ReturnType::Type(_, ty) => format!("{}", quote! { #ty }),
                },
            });
        }
    }

    let trait_name_string = trait_ident.to_string();
    let trait_schema = trait_schema::TraitSchema {
        name: trait_name_string,
        functions: trait_functions,
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
