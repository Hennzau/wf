use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

use crate::wired::{
    Struct,
    r#struct::{
        inner::inner,
        outer::{Outer, outer},
    },
    utils::{FlagIdent, SlotIdent},
};

pub(crate) mod classify;

pub mod inner;
pub mod outer;

pub fn expand(input: Struct) -> syn::Result<TokenStream> {
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

    let Outer {
        hty,
        consts,
        hc,
        hf,
        lf,
        ef,
        df,
    } = outer(&input.r#struct)?;

    let fields = input
        .data
        .take_struct()
        .expect("supports(struct_*) guarantees a struct");

    let mut len = Vec::<TokenStream>::with_capacity(fields.len());
    let mut header = Vec::<TokenStream>::with_capacity(fields.len());
    let mut encode = Vec::<TokenStream>::with_capacity(fields.len());
    let mut decode = Vec::<TokenStream>::with_capacity(fields.len());
    let mut members = Vec::<TokenStream>::with_capacity(fields.len());
    let mut asserts = Vec::<TokenStream>::with_capacity(fields.len());
    let mut constraints = Vec::<TokenStream>::with_capacity(fields.len());
    let mut masks = Vec::<TokenStream>::with_capacity(fields.len());

    let mut used_flags = HashSet::<FlagIdent>::new();
    let mut used_slots = HashSet::<SlotIdent>::new();

    let mut remaining = fields.len();
    for (i, field) in fields.into_iter().enumerate() {
        remaining -= 1;

        let field = inner::Field::new(field, i)?;
        let inner = inner(
            field,
            &lt,
            &hty,
            remaining == 0,
            &mut used_flags,
            &mut used_slots,
        )?;

        len.push(inner.len);
        header.push(inner.header);
        encode.push(inner.encode);
        decode.push(inner.decode);
        members.push(inner.member);

        if let Some(assert) = inner.assert {
            asserts.push(assert);
        }

        if let Some(constraint) = inner.constraint {
            constraints.push(constraint);
        }

        if let Some(mask) = inner.mask {
            masks.push(mask);
        }
    }

    let where_clause: WhereClause = where_clause
        .cloned()
        .map(|w| parse_quote!(#w #(#constraints),*))
        .unwrap_or(parse_quote!(where #(#constraints),*));

    let mask = if masks.is_empty() {
        quote!(pub const WF_MASK_HEADER: #hty = Self::WF_OWN_MASK_HEADER;)
    } else {
        quote!(pub const WF_MASK_HEADER: #hty = Self::WF_OWN_MASK_HEADER | #(#masks |)* 0;)
    };

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #consts
            #mask

            const fn __elvwf_asserts() {
                const {
                    let mut mh = Self::WF_OWN_MASK_HEADER;
                    #(#asserts)*
                }
            }
        }

        #[allow(unused_braces)]
        impl #impl_generics ::elvwf::Wired<#lt> for #ident #ty_generics #where_clause {
            type Header = #hty;
            type HeaderCodec = #hc;

            fn body_len(&self) -> usize {
                Self::__elvwf_asserts();

                let mut len: usize = 0;
                #(#len)*
                #lf
                len
            }

            fn header(&self) -> Result<Self::Header, ::elvwf::Error> {
                Self::__elvwf_asserts();

                let mut h: Self::Header = Self::WF_HEADER;
                #(#header)*
                #hf
                Ok(h)
            }

            fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), ::elvwf::Error> {
                Self::__elvwf_asserts();

                let _bs: usize = buf.len();
                #(#encode)*
                #ef
                Ok(())
            }

            fn decode_body(buf: &mut &#lt[u8], len: usize, h: Self::Header) -> Result<Self, ::elvwf::Error> {
                Self::__elvwf_asserts();

                let _bs: usize = buf.len();
                #(#decode)*
                #df
                Ok(Self { #(#members),* })
            }
        }
    })
}
