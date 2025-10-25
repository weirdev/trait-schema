use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, Receiver, ReturnType, TraitItem, Type, parse_macro_input};

use trait_schema_types as trait_schema;

#[proc_macro_attribute]
pub fn trait_schema(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let trait_ident = input.ident.clone();
    // Generated schema function name
    let schema_fn_ident = format_ident!("{}_schema", trait_ident);

    let mut trait_functions: Vec<trait_schema::FunctionSchema> = Vec::new();

    for it in &input.items {
        if let TraitItem::Fn(m) = it {
            let sig = &m.sig;

            trait_functions.push(trait_schema::FunctionSchema {
                name: sig.ident.to_string(),
                args: sig
                    .inputs
                    .iter()
                    // TODO: Skipping the self argument for now
                    .skip(1)
                    .filter_map(|arg| {
                        if let FnArg::Typed(pat_type) = arg {
                            if let Type::Reference(ty_ref) = &*pat_type.ty {
                                Some(format!("{}", quote! { #ty_ref }))
                            } else {
                                Some(format!("{}", quote! { #pat_type.ty }))
                            }
                        } else if let FnArg::Receiver(Receiver { .. }) = arg {
                            Some("self".to_string())
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

    output.into()
}
