use proc_macro2::TokenStream;
use quote::quote;

use crate::wired::{
    Enum,
    r#enum::{
        inner::{Variant, inner},
        outer::{Outer, outer},
    },
};

pub mod inner;
pub mod outer;

pub fn expand(input: Enum) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let Outer {
        header_ty,
        header_codec,
    } = outer(&input.r#enum)?;

    let variants = input
        .data
        .take_enum()
        .expect("supports(enum_*) guarantees an enum");

    let mut decode = Vec::<TokenStream>::with_capacity(variants.len());
    let mut mask_header = Vec::<TokenStream>::with_capacity(variants.len());

    for variant in variants {
        let variant = Variant::new(variant)?;
        let inner = inner(variant)?;

        decode.push(inner.decode);
        mask_header.push(inner.mask_header);
    }

    Ok(quote! {
        impl #ident {
            pub const WF_MASK_HEADER: #header_ty = #(#mask_header |)* 0;
        }

        #[allow(unused_braces)]
        impl<'a> ::elvwf::Wired<'a> for #ident where Self: Copy {
            type Header = #header_ty;
            type HeaderCodec = #header_codec;

            fn body_len(&self) -> usize {
                0
            }

            fn header(&self) -> Result<Self::Header, ::elvwf::Error> {
                ::elvwf::header::put::<#header_ty, _, _>(0, *self as #header_ty, #header_ty::MAX).map_err(Error::from)
            }

            fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), ::elvwf::Error> {
                Ok(())
            }

            fn decode_body(buf: &mut &'a [u8], len: usize, h: Self::Header) -> Result<Self, ::elvwf::Error> {
                match h {
                    #(#decode)*
                    _ => Err(::elvwf::Error::UnknownVariant)
                }
            }
        }
    })
}
