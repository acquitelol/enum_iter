use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Ident, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
};

struct ReprArgs {
    idents: Vec<Ident>,
}

impl Parse for ReprArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut idents = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            idents.push(ident);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(ReprArgs { idents })
    }
}

fn get_repr_idents(attr: &Attribute) -> Option<Vec<Ident>> {
    if attr.path().is_ident("repr") {
        let parse_result = attr.parse_args_with(ReprArgs::parse);
        if let Ok(args) = parse_result {
            return Some(args.idents);
        }
    }

    None
}

#[proc_macro_derive(Iter)]
pub fn codegen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let Data::Enum(data_enum) = input.data else {
        panic!("Cannot use the Iter derive macro on non-enum inputs");
    };

    let Some(attr) = input.attrs.iter().find(|x| x.path().is_ident("repr")) else {
        panic!(
            "Could not find a repr for this enum: you must define the `#repr(...)` attribute on the enum"
        );
    };

    let Some(repr) = get_repr_idents(attr).map(|idents| idents[0].clone()) else {
        panic!("Could not find the arguments of the `#repr(...)` attribute");
    };

    if repr == "C" {
        panic!(
            "Cannot handle `#[repr(C)]` enums: please use an explicit integer repr like `#[repr(u8)]`"
        );
    }

    let first = data_enum.variants.first().unwrap();
    let last = data_enum.variants.last().unwrap();

    let iter_ident_str = format!("{}{}", name, "__iter");
    let iter_ident: Ident = parse_str(&iter_ident_str).unwrap();

    let expanded = quote! {
        impl #name {
            fn iter() -> #iter_ident {
                #iter_ident {
                    inner: (#name::#first as #repr)..=(#name::#last as #repr),
                    _tag: std::marker::PhantomData::default(),
                }
            }
        }

        impl From<#repr> for #name {
            fn from(x: #repr) -> Self {
                unsafe { std::mem::transmute(x) }
            }
        }

        struct #iter_ident {
            inner: std::ops::RangeInclusive<#repr>,
            _tag: std::marker::PhantomData<#name>,
        }

        impl Iterator for #iter_ident {
            type Item = #name;

            fn next(&mut self) -> Option<#name> {
                self.inner.next().map(|x| #name::from(x))
            }
        }
    };

    TokenStream::from(expanded)
}
