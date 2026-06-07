use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

use crate::wired::{
    Union,
    union::{
        inner::{Variant, inner},
        outer::{Outer, outer},
    },
};

pub mod inner;
pub mod outer;

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

    let Outer {
        consts,
        header_ty,
        header_codec,
    } = outer(&input.r#union, &variants, &lt)?;

    let mut len = Vec::<TokenStream>::with_capacity(variants.len());
    let mut header = Vec::<TokenStream>::with_capacity(variants.len());
    let mut encode = Vec::<TokenStream>::with_capacity(variants.len());
    let mut decode = Vec::<TokenStream>::with_capacity(variants.len());
    let mut asserts = Vec::<TokenStream>::with_capacity(variants.len());
    let mut constraints = Vec::<TokenStream>::with_capacity(variants.len());

    for (seen, variant) in variants.into_iter().enumerate() {
        let variant = Variant::new(variant)?;
        let inner = inner(
            variant,
            &input.r#union.discriminant,
            &lt,
            &header_ty,
            seen == 0,
        )?;

        len.push(inner.len);
        header.push(inner.header);
        encode.push(inner.encode);
        decode.push(inner.decode);

        if let Some((assert, constraint)) = inner.constraints {
            asserts.push(assert);
            constraints.push(constraint);
        }
    }

    let where_clause: WhereClause = where_clause
        .cloned()
        .map(|w| parse_quote!(#w #(#constraints),*))
        .unwrap_or(parse_quote!(where #(#constraints),*));

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #consts

            const fn __elvwf_asserts() {
                const {
                    #(#asserts)*
                }
            }
        }

        impl #impl_generics ::elvwf::Wired<#lt> for #ident #ty_generics #where_clause {
            type Header = #header_ty;
            type HeaderCodec = #header_codec;

            fn body_len(&self) -> usize {
                match self {
                    #(#len),*
                }
            }

            fn header(&self) -> Result<Self::Header, ::elvwf::Error> {
                match self {
                    #(#header),*
                }
            }

            fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), ::elvwf::Error> {
                match self {
                    #(#encode),*
                }
            }

            fn decode_body(buf: &mut &#lt[u8], len: usize, h: Self::Header) -> Result<Self, ::elvwf::Error> {
                match ::elvwf::header::get::<_, _, Self::Header>(h, Self::DISCRIMINANT)? {
                    #(#decode),*,
                    _ => Err(elvwf::Error::UnknownVariant),
                }
            }
        }
    })
}
