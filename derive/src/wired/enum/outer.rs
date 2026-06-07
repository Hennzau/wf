use darling::*;
use proc_macro2::*;
use quote::*;

use super::super::utils::FixedFormat;

#[derive(Debug, FromMeta)]
pub(crate) struct EnumParams {
    repr: ReprParams,
    #[darling(default)]
    format: FixedFormat,
}

#[derive(Debug, FromMeta)]
pub enum ReprParams {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl ToTokens for ReprParams {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Self::U8 => quote!(u8),
            Self::U16 => quote!(u16),
            Self::U32 => quote!(u32),
            Self::U64 => quote!(u64),
            Self::I8 => quote!(i8),
            Self::I16 => quote!(i16),
            Self::I32 => quote!(i32),
            Self::I64 => quote!(i64),
        };
        tokens.extend(ts);
    }
}

pub struct Outer {
    pub header_ty: TokenStream,
    pub header_codec: TokenStream,
}

pub fn outer(input: &EnumParams) -> syn::Result<Outer> {
    let ty = &input.repr;
    let format = &input.format;

    Ok(Outer {
        header_ty: quote!(#ty),
        header_codec: quote!(#format),
    })
}
