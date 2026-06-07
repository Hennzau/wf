use darling::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::wired::{
    union::inner::WiredVariant,
    utils::{MaskIdent, to_turbofish},
};

#[derive(Debug, FromMeta)]
pub(crate) struct UnionParams {
    pub discriminant: MaskIdent,
}

pub struct Outer {
    pub consts: TokenStream,
    pub header_ty: TokenStream,
    pub header_codec: TokenStream,
}

pub fn outer(input: &UnionParams, variants: &[WiredVariant], lt: &Lifetime) -> syn::Result<Outer> {
    let Some(first_variant) = variants.first() else {
        return Err(syn::Error::new(
            Span::mixed_site(),
            "`#[wf(union(..))]` requires at least one tuple variant",
        ));
    };

    let Some(ty) = first_variant.fields.fields.first() else {
        return Err(syn::Error::new(
            first_variant.ident.span(),
            "`#[wf(union(..))]` requires tuple variants with one type",
        ));
    };

    let tty = to_turbofish(ty);
    let discriminant = &input.discriminant;

    Ok(Outer {
        consts: quote!(const DISCRIMINANT: <#ty as ::elvwf::Wired<#lt>>::Header = #tty::#discriminant;),
        header_ty: quote!(<#ty as ::elvwf::Wired<#lt>>::Header),
        header_codec: quote!(<#ty as ::elvwf::Wired<#lt>>::HeaderCodec),
    })
}
