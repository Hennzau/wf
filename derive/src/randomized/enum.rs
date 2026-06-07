use proc_macro2::TokenStream;
use quote::quote;

use crate::wired::Enum;

pub fn expand(input: Enum) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let variants = input
        .data
        .take_enum()
        .expect("supports(enum_*) guarantees an enum");

    let choices = variants.len();

    let mut random = Vec::<TokenStream>::with_capacity(choices);

    for (i, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;
        random.push(quote!(#i => Self:: #ident));
    }

    Ok(quote!(
        impl<'a> ::elvwf::Randomized<'a> for #ident {
            fn randomized(_: &mut &'a mut [u8]) -> Result<Self, ::elvwf::Error> {
                Ok(match ::elvwf::rand::random_range(0..#choices) {
                    #(#random,)*
                    _ => unreachable!(),
                })
            }
        }
    ))
}
