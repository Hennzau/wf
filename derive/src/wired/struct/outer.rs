use super::super::utils::FixedFormat;
use darling::FromMeta;
use proc_macro2::*;
use quote::*;
use std::str::FromStr;
use syn::*;

#[derive(Debug, Default, FromMeta)]
pub(crate) struct StructParams {
    align: Option<Align>,
    header: Option<Header>,
}

#[derive(Debug, FromMeta)]
pub(crate) enum Align {
    U16,
    U32,
    U64,
}

impl ToTokens for Align {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Self::U16 => quote!(u16),
            Self::U32 => quote!(u32),
            Self::U64 => quote!(u64),
        };
        tokens.extend(ts);
    }
}

#[derive(Debug, FromMeta)]
pub(crate) struct Header {
    dsl: String,
    format: Option<FixedFormat>,
    offset: Option<Expr>,
}

pub struct Outer {
    pub hty: TokenStream,
    pub consts: TokenStream,
    pub hc: TokenStream,
    pub hf: TokenStream,
    pub lf: TokenStream,
    pub ef: TokenStream,
    pub df: TokenStream,
}

pub fn outer(input: &StructParams) -> Result<Outer> {
    let (hty, consts, hc, hf) = input
        .header
        .as_ref()
        .map(|h| {
            consts_hty(&h.dsl).map(|(hty, consts)| {
                (
                    hty,
                    consts,
                    h.format
                        .as_ref()
                        .map(|f| quote!(#f))
                        .unwrap_or_else(|| quote!(::elvwf::Be)),
                    h.offset
                        .as_ref()
                        .map(|o| quote!(h += #o;))
                        .unwrap_or_else(|| quote!()),
                )
            })
        })
        .transpose()?
        .unwrap_or_else(|| {
            (
                quote!(()),
                quote!(
                    const WF_HEADER: () = ();
                    const WF_OWN_MASK_HEADER: () = ();
                ),
                quote!(elvwf::Be),
                quote!(),
            )
        });

    let (lf, ef, df) = input
        .align.as_ref()
        .map(|a| {
            (
                quote!(len += (::core::mem::size_of::<#a>() - len % ::core::mem::size_of::<#a>()) % ::core::mem::size_of::<#a>();),
                quote!(::elvwf::slice::noprefix::encode::<&[u8]>(buf, &(&[0; 8])[..(::core::mem::size_of::<#a>() - (_bs - buf.len()) % ::core::mem::size_of::<#a>()) % ::core::mem::size_of::<#a>()])?;),
                quote!(let _ = ::elvwf::slice::noprefix::decode::<&[u8]>(buf, (::core::mem::size_of::<#a>() - (_bs - buf.len()) % ::core::mem::size_of::<#a>()) % ::core::mem::size_of::<#a>() as usize)?;),
            )
        })
        .unwrap_or_else(|| (quote!(), quote!(), quote!()));

    Ok(Outer {
        hty,
        consts,
        hc,
        hf,
        lf,
        ef,
        df,
    })
}

fn header_slot(field: &str) -> Result<(Option<String>, u8, Option<&str>)> {
    let (name, rest) = field
        .split_once(':')
        .map_or((field, None), |(n, r)| (n.trim(), Some(r.trim())));
    let (bits, value) = match rest {
        Some(r) => match r.split_once('=') {
            Some((s, v)) => (
                s.trim().parse().map_err(|_| {
                    Error::new(Span::call_site(), "Bit length should be an integer")
                })?,
                Some(v.trim()),
            ),
            None => (
                r.parse().map_err(|_| {
                    Error::new(Span::call_site(), "Bit length should be an integer")
                })?,
                None,
            ),
        },
        None => (1, None),
    };
    let name = (name != "_").then(|| name.to_uppercase());
    Ok((name, bits, value))
}

fn consts_hty(input: &str) -> Result<(TokenStream, TokenStream)> {
    let fields = input
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(header_slot)
        .collect::<Result<Vec<(Option<String>, u8, Option<&str>)>>>()?;

    let total: usize = fields.iter().map(|(_, b, _)| *b as usize).sum();
    let width = total + 2;

    if !total.is_power_of_two() || !(8..=64).contains(&total) {
        return Err(Error::new(
            Span::call_site(),
            "Header len should be a power of 2, between 8 and 64",
        ));
    }

    let hty = format_ident!("u{total}");

    let mut consts = Vec::<TokenStream>::with_capacity(fields.len());
    let mut header = Vec::<TokenStream>::with_capacity(fields.len());
    let mut mask_header = Vec::<TokenStream>::with_capacity(fields.len());

    let mut count = 0u8;
    for (name, bits, value) in fields.iter().rev() {
        let mask = (((1u128 << bits) - 1) << count) as u64;
        let formatted = format!("{mask:#0width$b}");
        let formatted = TokenStream::from_str(&formatted)?;
        match (name, value) {
            (Some(name), Some(v)) => {
                let v = TokenStream::from_str(v)?;
                let name = Ident::from_string(name)?;
                let mname = format_ident!("WF_MASK_{name}");

                header.push(quote!((Self::#name << Self::#mname.trailing_zeros())));
                mask_header.push(quote!(Self::#mname));
                consts.push(quote!(const #name: #hty = #v;));
                consts.push(quote!(const #mname: #hty = #formatted;));
            }
            (Some(name), None) => {
                let mname = if *bits == 1 {
                    format_ident!("WF_FLAG_{name}")
                } else {
                    format_ident!("WF_SLOT_{name}")
                };
                mask_header.push(quote!(Self::#mname));
                consts.push(quote!(const #mname: #hty = #formatted;));
            }
            (None, Some(_)) => {
                return Err(Error::new(
                    Span::call_site(),
                    "Value should always be named",
                ));
            }
            _ => {}
        }
        count += bits;
    }

    Ok((
        quote!(#hty),
        quote! {
            #(#consts)*
            const WF_HEADER: #hty = #(#header+)* 0;
            const WF_OWN_MASK_HEADER: #hty = #(#mask_header |)* 0;
        },
    ))
}
