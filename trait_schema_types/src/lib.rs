use std::fmt::Display;

use quote::quote;
use syn::{punctuated::Punctuated, token::Comma};

// Needed so macro type references work correctly
#[allow(unused_imports)]
use crate as trait_schema;

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
                    ::trait_schema::TraitSchema {
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
    pub args: Vec<FunctionArgSchema>,
    pub return_type: String,
    pub body: Option<String>,
}

impl Into<proc_macro2::TokenStream> for FunctionSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let args_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .args
            .into_iter()
            .map(|arg| Into::<proc_macro2::TokenStream>::into(arg))
            .collect::<Punctuated<_, Comma>>();
        let return_type_lit = proc_macro2::Literal::string(&self.return_type);

        let body = if let Some(body) = self.body {
            let body_lit = proc_macro2::Literal::string(&body);
            quote! {
                Some(::std::string::String::from(#body_lit))
            }
        } else {
            quote! {
                None
            }
        };

        quote! {
            ::trait_schema::FunctionSchema {
                name: ::std::string::String::from(#name_lit),
                args: ::std::vec![
                    #args_tokens
                ],
                return_type: ::std::string::String::from(#return_type_lit),
                body: #body,
            }
        }
    }
}

impl Display for FunctionSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if let Some(ty) = &arg.ty {
                write!(f, "{}: {}", arg.name, ty)?;
            } else {
                write!(f, "{}", arg.name)?;
            }
        }
        write!(f, ")")?;
        write!(f, " -> {}", self.return_type)?;
        if let Some(body) = &self.body {
            write!(f, " {{ {} }}", body)?;
        } else {
            write!(f, ";")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FunctionArgSchema {
    pub name: String,
    pub ty: Option<String>,
    pub annotations: Option<FnArgAnnotations>,
}

impl Into<proc_macro2::TokenStream> for FunctionArgSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let ty_lit = self.ty.as_ref().map(|s| proc_macro2::Literal::string(s));
        let annotations_tokens = if let Some(annotations) = self.annotations {
            let annotations_ts: proc_macro2::TokenStream = annotations.into();
            quote! { Some(#annotations_ts) }
        } else {
            quote! { None }
        };

        let ty_tokens: proc_macro2::TokenStream = if let Some(ty_lit) = ty_lit {
            quote! { ty: ::std::option::Option::Some(::std::string::String::from(#ty_lit)), }
        } else {
            quote! { ty: ::std::option::Option::None, }
        };

        quote! {
            ::trait_schema::FunctionArgSchema {
            name: ::std::string::String::from(#name_lit),
            #ty_tokens
            annotations: #annotations_tokens,
            }
        }
    }
}

#[derive(Debug)]
pub struct FnArgAnnotations {
    // Examples of supported args:
    //   #[arg(collection_as_item, assert_len = 1)]
    pub collection_as_item: bool,
    pub assert_len: Option<usize>, // e.g. 10
}

impl Into<proc_macro2::TokenStream> for FnArgAnnotations {
    fn into(self) -> proc_macro2::TokenStream {
        let collection_as_item = self.collection_as_item;
        let assert_len = match self.assert_len {
            Some(len) => quote! { Some(#len) },
            None => quote! { None },
        };
        quote! {
            ::trait_schema::FnArgAnnotations {
                collection_as_item: #collection_as_item,
                assert_len: #assert_len,
            }
        }
    }
}

impl FnArgAnnotations {
    pub fn new() -> Self {
        FnArgAnnotations {
            collection_as_item: false,
            assert_len: None,
        }
    }
}
