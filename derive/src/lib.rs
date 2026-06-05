use darling::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, *};

use crate::{classify::classify, header::parse_header};

pub(crate) mod classify;
pub(crate) mod header;
pub(crate) mod parser2;

#[derive(Debug, FromMeta)]
pub(crate) enum LenSpec {
    Prefixed { format: Ident },
    Slot(Ident),
    Remaining,
    Embedded,
}

#[derive(Debug, FromMeta)]
pub(crate) struct OptSpec {
    #[darling(rename = "if")]
    pub condition: Option<Expr>,
    #[darling(flatten)]
    pub trigger: TriggerSource,
}

#[derive(Debug, FromMeta)]
pub(crate) enum TriggerSource {
    Flag(Ident),
}

#[derive(Debug, FromField)]
#[darling(attributes(wf))]
pub(crate) struct WiredField {
    ident: Option<Ident>,
    ty: Type,

    pub len: Option<LenSpec>,
    pub opt: Option<OptSpec>,
    pub format: Option<Ident>,
}

impl WiredField {
    pub fn no_len(&self) -> syn::Result<()> {
        if self.len.is_some() {
            return Err(syn::Error::new(
                self.ty.span(),
                "This type must not have a `len` attribute",
            ));
        }

        Ok(())
    }

    pub fn no_format(&self) -> syn::Result<()> {
        if self.format.is_some() {
            return Err(syn::Error::new(
                self.ty.span(),
                "This type must not have a `format` attribute",
            ));
        }

        Ok(())
    }

    pub fn opt_condition(&self) -> syn::Result<(&Expr, &TriggerSource)> {
        self.opt
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.ty.span(), "This type requires an `opt` attribute"))
            .map(|opt| {
                let OptSpec {
                    condition: Some(condition),
                    trigger,
                } = opt
                else {
                    return Err(syn::Error::new(
                        self.ty.span(),
                        "This type must not have an `opt(if = ..)` attribute",
                    ));
                };

                Ok((condition, trigger))
            })
            .flatten()
    }

    pub fn opt_no_condition(&self) -> syn::Result<&TriggerSource> {
        self.opt
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.ty.span(), "This type requires an `opt` attribute"))
            .map(|opt| {
                if opt.condition.is_none() {
                    Ok::<_, syn::Error>(&opt.trigger)
                } else {
                    Err(syn::Error::new(
                        self.ty.span(),
                        "This type must not have an `opt(if = ..)` attribute",
                    ))
                }
            })
            .flatten()
    }

    pub fn len(&self) -> syn::Result<&LenSpec> {
        self.len
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.ty.span(), "This type requires a `len` attribute"))
    }

    pub fn format(&self) -> syn::Result<&Ident> {
        self.format.as_ref().ok_or_else(|| {
            syn::Error::new(self.ty.span(), "This type requires a `format` attribute")
        })
    }
}

#[derive(Debug, FromMeta)]
pub(crate) struct HeaderSpec {
    dsl: String,
    format: Option<Ident>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wf), supports(struct_named, struct_unit))]
pub(crate) struct WiredInput {
    ident: Ident,
    generics: Generics,
    data: ast::Data<darling::util::Ignored, WiredField>,
    header: Option<HeaderSpec>,
}

#[proc_macro_derive(WiredBlank, attributes(wf))]
pub fn derive_wired_blank(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote!().into()
}

#[proc_macro_derive(Wired, attributes(wf))]
pub fn derive_wired(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let parsed = match WiredInput::from_derive_input(&input) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    match expand(&parsed) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(input: &WiredInput) -> syn::Result<TokenStream> {
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

    let fields = input
        .data
        .as_ref()
        .take_struct()
        .expect("supports(struct_named) guarantees a struct");

    let ((hty, consts), hc) = if let Some(h) = &input.header {
        let hc = if let Some(fmt) = &h.format {
            quote!(elvwf::scalar::#fmt)
        } else {
            quote!(elvwf::scalar::Be)
        };

        (parse_header(&h.dsl)?, hc)
    } else {
        ((quote!(()), quote!()), quote!(elvwf::scalar::Be))
    };

    let mut len = Vec::<TokenStream>::with_capacity(fields.len());
    let mut header = Vec::<TokenStream>::with_capacity(fields.len());
    let mut encode = Vec::<TokenStream>::with_capacity(fields.len());
    let mut decode = Vec::<TokenStream>::with_capacity(fields.len());
    let mut idents = Vec::<_>::with_capacity(fields.len());

    let mut remaining = fields.len();
    for f in fields.iter() {
        remaining -= 1;
        let kind = classify(&f.ty)?;
        let parsed = parser2::ParsedField::new(f, kind, remaining == 0)?;

        len.push(parsed.len);
        header.push(parsed.header);
        encode.push(parsed.encode);
        decode.push(parsed.decode);
        idents.push(parsed.ident);
    }

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #consts
        }

        #[allow(unused_braces)]
        impl #impl_generics elvwf::Wired<#lt> for #ident #ty_generics #where_clause {
            type Header = #hty;
            type HeaderCodec = #hc;

            fn body_len(&self) -> usize {
                let mut len: usize = 0;
                #(#len)*
                len
            }

            fn header(&self) -> Result<Self::Header, elvwf::Error> {
                let mut h: Self::Header = elvwf::header::zero();
                #(#header)*
                Ok(h)
            }

            fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), elvwf::Error> {
                #(#encode)*
                Ok(())
            }

            fn decode_body(buf: &mut &#lt[u8], len: usize, h: Self::Header) -> Result<Self, elvwf::Error> {
                let buf_start: usize = buf.len();

                #(#decode)*

                Ok(Self {
                    #(#idents),*
                })
            }
        }
    })
}
