use darling::*;
use proc_macro2::*;
use quote::*;
use syn::*;

use super::super::utils::{MaskIdent, strip_lifetimes, to_turbofish};

#[derive(Debug, FromVariant)]
pub struct WiredVariant {
    pub ident: Ident,
    pub fields: darling::ast::Fields<Type>,
}

pub struct Variant {
    pub ident: Ident,
    pub ty: Type,
}

impl Variant {
    pub fn new(variant: WiredVariant) -> syn::Result<Self> {
        let Some(ty) = variant.fields.fields.first().cloned() else {
            return Err(syn::Error::new(
                variant.ident.span(),
                "`#[wf(union(..))]` requires tuple variants with one type",
            ));
        };

        Ok(Self {
            ident: variant.ident,
            ty,
        })
    }
}

pub struct Inner {
    pub len: TokenStream,
    pub header: TokenStream,
    pub encode: TokenStream,
    pub decode: TokenStream,
    pub constraints: Option<(TokenStream, TokenStream)>,
}

pub fn inner(
    input: Variant,
    mdisc: &MaskIdent,
    lt: &Lifetime,
    header_ty: &TokenStream,
    first: bool,
) -> syn::Result<Inner> {
    let ident = &input.ident;
    let ty = &input.ty;
    let sty = strip_lifetimes(&input.ty);
    let tty = to_turbofish(&input.ty);
    let disc = &mdisc.0;

    let len = quote!(Self::#ident(x) => ::elvwf::msg::noprefixnohead::len::<#sty>(x));
    let header = quote!(Self::#ident(x) => ::elvwf::msg::header::<#sty>(x));
    let encode = quote!(Self::#ident(x) => ::elvwf::msg::noprefixnohead::encode::<#sty>(buf, x));
    let decode = quote!(#tty::#disc => ::elvwf::msg::noprefixnohead::decode::<#sty>(buf, h, len).map(Self::#ident));

    let constraints = if !first {
        Some((
            quote!(assert!(#tty::#mdisc == Self::DISCRIMINANT);),
            quote!(#ty: ::elvwf::Wired<#lt, Header = #header_ty>),
        ))
    } else {
        None
    };

    Ok(Inner {
        len,
        header,
        encode,
        decode,
        constraints,
    })
}
