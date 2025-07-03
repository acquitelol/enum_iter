use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Ident, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
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

    let expanded = quote! {
        impl #name {
            fn iter() -> core::iter::Map<core::ops::RangeInclusive<#repr>, impl FnMut(#repr) -> #name> {
                ((#name::#first as #repr)..=(#name::#last as #repr))
                    .map(|x| unsafe { core::mem::transmute::<#repr, #name>(x) })
            }
        }
    };

    TokenStream::from(expanded)
}
