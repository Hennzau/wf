use darling::*;
use proc_macro2::*;
use quote::*;
use syn::*;

#[derive(Debug, FromVariant)]
pub struct WiredVariant {
    pub ident: Ident,
    pub discriminant: Option<Expr>,
}

#[derive(Debug)]
pub struct Variant {
    pub ident: Ident,
    pub discriminant: LitInt,
}

impl Variant {
    pub fn new(variant: WiredVariant) -> syn::Result<Variant> {
        let Some(Expr::Lit(ExprLit {
            lit: Lit::Int(discriminant),
            ..
        })) = variant.discriminant
        else {
            return Err(syn::Error::new(
                variant.ident.span(),
                "`#[wf(enum(..))]` requires int literal discriminant on each variant",
            ));
        };

        Ok(Variant {
            ident: variant.ident,
            discriminant,
        })
    }
}

pub struct Inner {
    pub decode: TokenStream,
    pub mask_header: TokenStream,
}

pub fn inner(input: Variant) -> syn::Result<Inner> {
    let ident = &input.ident;
    let discriminant = &input.discriminant;

    let decode = quote!(#discriminant => Ok(Self::#ident),);
    let mask_header = quote!(#discriminant);

    Ok(Inner {
        decode,
        mask_header,
    })
}
