use quote::{format_ident, quote};
use syn::{
    FnArg, ItemTrait, Receiver, ReturnType, TraitItem, Type, parse_macro_input,
    punctuated::Punctuated, token::Comma,
};

#[derive(Debug)]
pub struct TraitSchema {
    pub name: String,
    pub functions: Vec<FunctionSchema>,
}

impl Into<proc_macro2::TokenStream> for TraitSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let field_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .functions
            .into_iter()
            .map(|f| Into::<proc_macro2::TokenStream>::into(f))
            .collect::<Punctuated<_, Comma>>();

        quote! {
            {
                let functions = ::std::vec![
                    #field_tokens
                ];
                    ::trait_schema_types::TraitSchema {
                        name: ::std::string::String::from(#name_lit),
                        functions: functions,
                    }
            }
        }
    }
}

#[derive(Debug)]
pub struct FunctionSchema {
    pub name: String,
    pub args: Vec<String>,
    pub return_type: String,
}

impl Into<proc_macro2::TokenStream> for FunctionSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let args_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .args
            .into_iter()
            .map(|arg| {
                let arg_lit = proc_macro2::Literal::string(&arg);
                quote! {
                    ::std::string::String::from(#arg_lit)
                }
            })
            .collect::<Punctuated<_, Comma>>();
        let return_type_lit = proc_macro2::Literal::string(&self.return_type);
        quote! {
            ::trait_schema_types::FunctionSchema {
                name: ::std::string::String::from(#name_lit),
                args: ::std::vec![
                    #args_tokens
                ],
                return_type: ::std::string::String::from(#return_type_lit),
            }
        }
    }
}
