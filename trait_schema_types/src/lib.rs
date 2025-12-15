use std::fmt::Display;

use quote::quote;
use syn::{punctuated::Punctuated, token::Comma};

// Needed so macro type references work correctly
#[allow(unused_imports)]
use crate as trait_schema;

#[derive(Debug)]
pub struct GenericParamAnnotations {
    pub cffi_type: Option<String>,
}

impl Into<proc_macro2::TokenStream> for GenericParamAnnotations {
    fn into(self) -> proc_macro2::TokenStream {
        let cffi_type = self
            .cffi_type
            .as_ref()
            .map(|s| proc_macro2::Literal::string(s))
            .map(|lit| quote! { Some(::std::string::String::from(#lit)) })
            .unwrap_or(quote! { None });
        quote! {
            ::trait_schema::GenericParamAnnotations {
                cffi_type: #cffi_type,
            }
        }
    }
}

impl GenericParamAnnotations {
    pub fn new() -> Self {
        GenericParamAnnotations { cffi_type: None }
    }
}

#[derive(Debug)]
pub struct GenericParamSchema {
    pub name: String,
    pub annotations: Option<GenericParamAnnotations>,
}

impl Into<proc_macro2::TokenStream> for GenericParamSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let annotations_tokens = if let Some(annotations) = self.annotations {
            let annotations_ts: proc_macro2::TokenStream = annotations.into();
            quote! { Some(#annotations_ts) }
        } else {
            quote! { None }
        };

        quote! {
            ::trait_schema::GenericParamSchema {
                name: ::std::string::String::from(#name_lit),
                annotations: #annotations_tokens,
            }
        }
    }
}

#[derive(Debug)]
pub struct TraitSchema {
    pub name: String,
    pub functions: Vec<FunctionSchema>,
    pub generics: Vec<GenericParamSchema>,
    pub supertraits: Vec<String>,
}

impl Into<proc_macro2::TokenStream> for TraitSchema {
    fn into(self) -> proc_macro2::TokenStream {
        let name_lit = proc_macro2::Literal::string(&self.name);
        let field_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .functions
            .into_iter()
            .map(|f| Into::<proc_macro2::TokenStream>::into(f))
            .collect::<Punctuated<_, Comma>>();

        let generics_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .generics
            .into_iter()
            .map(|g| Into::<proc_macro2::TokenStream>::into(g))
            .collect::<Punctuated<_, Comma>>();

        let supertraits_tokens: Punctuated<proc_macro2::TokenStream, Comma> = self
            .supertraits
            .into_iter()
            .map(|s| {
                let s_lit = proc_macro2::Literal::string(&s);
                quote! { ::std::string::String::from(#s_lit) }
            })
            .collect::<Punctuated<_, Comma>>();

        quote! {
            {
                let functions = ::std::vec![
                    #field_tokens
                ];
                let generics = ::std::vec![
                    #generics_tokens
                ];
                let supertraits = ::std::vec![
                    #supertraits_tokens
                ];
                    ::trait_schema::TraitSchema {
                        name: ::std::string::String::from(#name_lit),
                        functions: functions,
                        generics: generics,
                        supertraits: supertraits,
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
    pub extern_layout: Option<String>,
    pub annotations: Option<FunctionAnnotations>,
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

        let extern_layout = if let Some(extern_layout) = self.extern_layout {
            let extern_layout = proc_macro2::Literal::string(&extern_layout);
            quote! {
                Some(::std::string::String::from(#extern_layout))
            }
        } else {
            quote! {
                None
            }
        };

        let annotations_tokens = if let Some(annotations) = self.annotations {
            let annotations_ts: proc_macro2::TokenStream = annotations.into();
            quote! { Some(#annotations_ts) }
        } else {
            quote! { None }
        };

        quote! {
            ::trait_schema::FunctionSchema {
                name: ::std::string::String::from(#name_lit),
                args: ::std::vec![
                    #args_tokens
                ],
                return_type: ::std::string::String::from(#return_type_lit),
                body: #body,
                extern_layout: #extern_layout,
                annotations: #annotations_tokens,
            }
        }
    }
}

impl Display for FunctionSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(extern_layout) = &self.extern_layout {
            write!(f, "extern \"{}\" ", extern_layout)?;
        }
        write!(f, "fn {}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if let Some(ty) = &arg.ty {
                write!(f, "{}: {}", arg.name, ty)?;
            } else {
                // Only self param lack an explicit type
                // TODO: Assuming self param is reference
                write!(f, "&{}", arg.name)?;
            }
        }
        write!(f, ")")?;
        write!(f, " -> {}", self.return_type)?;
        if let Some(body) = &self.body {
            write!(f, " {{\n{}\n}}", body)?;
        } else {
            write!(f, ";")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FunctionAnnotations {
    pub cffi_impl_no_op: bool,
}

impl Into<proc_macro2::TokenStream> for FunctionAnnotations {
    fn into(self) -> proc_macro2::TokenStream {
        let cffi_impl_no_op = self.cffi_impl_no_op;
        quote! {
            ::trait_schema::FunctionAnnotations {
                cffi_impl_no_op: #cffi_impl_no_op,
            }
        }
    }
}

impl FunctionAnnotations {
    pub fn new() -> Self {
        FunctionAnnotations { cffi_impl_no_op: false }
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
    pub cffi_type: Option<String>, // ptr<i32>, opt_ptr<f32>, etc.
}

impl Into<proc_macro2::TokenStream> for FnArgAnnotations {
    fn into(self) -> proc_macro2::TokenStream {
        let collection_as_item = self.collection_as_item;
        let cffi_type = self
            .cffi_type
            .as_ref()
            .map(|s| proc_macro2::Literal::string(s))
            .map(|lit| quote! { Some(::std::string::String::from(#lit)) })
            .unwrap_or(quote! { None });
        let assert_len = match self.assert_len {
            Some(len) => quote! { Some(#len) },
            None => quote! { None },
        };
        quote! {
            ::trait_schema::FnArgAnnotations {
                collection_as_item: #collection_as_item,
                assert_len: #assert_len,
                cffi_type: #cffi_type,
            }
        }
    }
}

impl FnArgAnnotations {
    pub fn new() -> Self {
        FnArgAnnotations {
            collection_as_item: false,
            assert_len: None,
            cffi_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_arg_annotations_new() {
        let annotations = FnArgAnnotations::new();
        assert!(!annotations.collection_as_item);
        assert!(annotations.assert_len.is_none());
    }

    #[test]
    fn test_fn_arg_annotations_creation_with_values() {
        let annotations = FnArgAnnotations {
            collection_as_item: true,
            assert_len: Some(42),
            cffi_type: Some("ptr<i32>".to_string()),
        };
        assert!(annotations.collection_as_item);
        assert_eq!(annotations.assert_len, Some(42));
        assert_eq!(annotations.cffi_type, Some("ptr<i32>".to_string()));
    }

    #[test]
    fn test_function_arg_schema_creation() {
        let arg = FunctionArgSchema {
            name: "test_arg".to_string(),
            ty: Some("String".to_string()),
            annotations: Some(FnArgAnnotations {
                collection_as_item: false,
                assert_len: Some(10),
                cffi_type: None,
            }),
        };

        assert_eq!(arg.name, "test_arg");
        assert_eq!(arg.ty, Some("String".to_string()));
        assert!(arg.annotations.is_some());
        if let Some(ann) = arg.annotations {
            assert_eq!(ann.assert_len, Some(10));
        }
    }

    #[test]
    fn test_function_arg_schema_without_annotations() {
        let arg = FunctionArgSchema {
            name: "simple_arg".to_string(),
            ty: Some("i32".to_string()),
            annotations: None,
        };

        assert_eq!(arg.name, "simple_arg");
        assert_eq!(arg.ty, Some("i32".to_string()));
        assert!(arg.annotations.is_none());
    }

    #[test]
    fn test_function_schema_creation() {
        let args = vec![
            FunctionArgSchema {
                name: "arg1".to_string(),
                ty: Some("String".to_string()),
                annotations: None,
            },
            FunctionArgSchema {
                name: "arg2".to_string(),
                ty: Some("i32".to_string()),
                annotations: None,
            },
        ];

        let func = FunctionSchema {
            name: "test_fn".to_string(),
            args,
            return_type: "bool".to_string(),
            body: None,
            extern_layout: None,
            annotations: None,
        };

        assert_eq!(func.name, "test_fn");
        assert_eq!(func.args.len(), 2);
        assert_eq!(func.return_type, "bool");
        assert!(func.body.is_none());
    }

    #[test]
    fn test_function_schema_display() {
        let args = vec![
            FunctionArgSchema {
                name: "x".to_string(),
                ty: Some("i32".to_string()),
                annotations: None,
            },
            FunctionArgSchema {
                name: "y".to_string(),
                ty: Some("String".to_string()),
                annotations: None,
            },
        ];

        let func = FunctionSchema {
            name: "my_function".to_string(),
            args,
            return_type: "bool".to_string(),
            body: None,
            extern_layout: None,
            annotations: None,
        };

        let display_str = format!("{}", func);
        assert!(display_str.contains("my_function"));
        assert!(display_str.contains("x: i32"));
        assert!(display_str.contains("y: String"));
        assert!(display_str.contains("-> bool"));
        assert!(display_str.contains(";"));
    }

    #[test]
    fn test_function_schema_display_with_body() {
        let args = vec![];
        let func = FunctionSchema {
            name: "with_body".to_string(),
            args,
            return_type: "String".to_string(),
            body: Some("return \"hello\".to_string();".to_string()),
            extern_layout: None,
            annotations: None,
        };

        let display_str = format!("{}", func);
        assert!(display_str.contains("with_body"));
        assert!(display_str.contains("hello"));
        assert!(display_str.contains("{"));
        assert!(display_str.contains("}"));
    }

    #[test]
    fn test_trait_schema_creation() {
        let functions = vec![
            FunctionSchema {
                name: "method1".to_string(),
                args: vec![],
                return_type: "()".to_string(),
                body: None,
                extern_layout: None,
                annotations: None,
            },
            FunctionSchema {
                name: "method2".to_string(),
                args: vec![FunctionArgSchema {
                    name: "arg".to_string(),
                    ty: Some("String".to_string()),
                    annotations: None,
                }],
                return_type: "String".to_string(),
                body: None,
                extern_layout: None,
                annotations: None,
            },
        ];

        let schema = TraitSchema {
            name: "MyTrait".to_string(),
            functions,
            generics: vec![],
            supertraits: vec![],
        };

        assert_eq!(schema.name, "MyTrait");
        assert_eq!(schema.functions.len(), 2);
        assert_eq!(schema.functions[0].name, "method1");
        assert_eq!(schema.functions[1].name, "method2");
        assert_eq!(schema.generics.len(), 0);
        assert_eq!(schema.supertraits.len(), 0);
    }

    #[test]
    fn test_function_schema_no_args() {
        let func = FunctionSchema {
            name: "no_args".to_string(),
            args: vec![],
            return_type: "()".to_string(),
            body: None,
            extern_layout: None,
            annotations: None,
        };

        assert_eq!(func.args.len(), 0);
        let display_str = format!("{}", func);
        assert!(display_str.contains("no_args()"));
    }

    #[test]
    fn test_function_arg_schema_no_type() {
        let arg = FunctionArgSchema {
            name: "self".to_string(),
            ty: None,
            annotations: None,
        };

        assert_eq!(arg.name, "self");
        assert!(arg.ty.is_none());
    }

    #[test]
    fn test_generic_param_annotations_new() {
        let annotations = GenericParamAnnotations::new();
        assert!(annotations.cffi_type.is_none());
    }

    #[test]
    fn test_generic_param_annotations_with_cffi_type() {
        let annotations = GenericParamAnnotations {
            cffi_type: Some("ptr<void>".to_string()),
        };
        assert_eq!(annotations.cffi_type, Some("ptr<void>".to_string()));
    }

    #[test]
    fn test_generic_param_schema_creation() {
        let param = GenericParamSchema {
            name: "T".to_string(),
            annotations: Some(GenericParamAnnotations {
                cffi_type: Some("ptr<i32>".to_string()),
            }),
        };

        assert_eq!(param.name, "T");
        assert!(param.annotations.is_some());
        if let Some(ann) = param.annotations {
            assert_eq!(ann.cffi_type, Some("ptr<i32>".to_string()));
        }
    }

    #[test]
    fn test_generic_param_schema_without_annotations() {
        let param = GenericParamSchema {
            name: "U".to_string(),
            annotations: None,
        };

        assert_eq!(param.name, "U");
        assert!(param.annotations.is_none());
    }

    #[test]
    fn test_trait_schema_with_generics() {
        let generics = vec![GenericParamSchema {
            name: "T".to_string(),
            annotations: Some(GenericParamAnnotations {
                cffi_type: Some("ptr<void>".to_string()),
            }),
        }];

        let schema = TraitSchema {
            name: "SpecializedTrait".to_string(),
            functions: vec![],
            generics,
            supertraits: vec![],
        };

        assert_eq!(schema.name, "SpecializedTrait");
        assert_eq!(schema.generics.len(), 1);
        assert_eq!(schema.generics[0].name, "T");
        assert_eq!(schema.supertraits.len(), 0);
    }

    #[test]
    fn test_trait_schema_with_supertraits() {
        let schema = TraitSchema {
            name: "MyTraitWithBase".to_string(),
            functions: vec![],
            generics: vec![],
            supertraits: vec![
                "BaseTrait".to_string(),
                "OtherTrait".to_string(),
            ],
        };

        assert_eq!(schema.name, "MyTraitWithBase");
        assert_eq!(schema.supertraits.len(), 2);
        assert_eq!(schema.supertraits[0], "BaseTrait");
        assert_eq!(schema.supertraits[1], "OtherTrait");
    }
}
