use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, Lifetime, LifetimeParam};

use crate::wired::{Union, union::inner::Variant, utils::strip_lifetimes};

pub fn expand(input: Union) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let mut generics = input.generics.clone();
    let lt = match generics.lifetimes().next() {
        Some(lt) => lt.lifetime.clone(),
        None => {
            let lt = Lifetime::new("'a", proc_macro2::Span::call_site());
            generics
                .params
                .insert(0, GenericParam::Lifetime(LifetimeParam::new(lt.clone())));
            lt
        }
    };
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let (impl_generics, _, _) = generics.split_for_impl();

    let variants = input
        .data
        .take_enum()
        .expect("supports(enum_*) guarantees an enum");

    let choices = variants.len();

    let mut random = Vec::<TokenStream>::with_capacity(choices);

    for (i, variant) in variants.into_iter().enumerate() {
        let variant = Variant::new(variant)?;
        let ident = variant.ident;
        let sty = strip_lifetimes(&variant.ty);
        random.push(quote!(#i => Self:: #ident(::elvwf::msg::randomized::<#sty>(buf)?)));
    }

    Ok(quote!(
        impl #impl_generics ::elvwf::Randomized<#lt> for #ident #ty_generics #where_clause {
            fn randomized(buf: &mut &'a mut [u8]) -> Result<Self, ::elvwf::Error> {
                Ok(match ::elvwf::rand::random_range(0..#choices) {
                    #(#random,)*
                    _ => unreachable!()
                })
            }
        }
    ))
}
