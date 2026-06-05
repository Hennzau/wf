use std::str::FromStr;

use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::*;

fn parse_field(field: &str) -> Result<(Option<String>, u8, Option<&str>)> {
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

pub fn parse_header(input: &str) -> Result<(TokenStream, TokenStream)> {
    let fields = input
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_field)
        .collect::<Result<Vec<(Option<String>, u8, Option<&str>)>>>()?;

    let total: usize = fields.iter().map(|(_, b, _)| *b as usize).sum();
    let width = total + 2;

    if !total.is_power_of_two() || total > 64 || total < 8 {
        return Err(Error::new(
            Span::call_site(),
            "Header len should be a power of 2, between 8 and 64",
        ));
    }

    let header = format_ident!("u{total}");

    let mut consts = Vec::<TokenStream>::with_capacity(fields.len());
    let mut count = 0u8;
    for (name, bits, value) in fields.iter().rev() {
        let mask = (((1u128 << bits) - 1) << count) as u64;
        let formatted = format!("{mask:#0width$b}");
        let formatted = TokenStream::from_str(&formatted)?;
        match (name, value) {
            (Some(name), Some(v)) => {
                let v = TokenStream::from_str(v)?;
                let name = Ident::from_string(name)?;
                let mname = format_ident!("MASK_{name}");

                consts.push(quote!(const #name: #header = #v;));
                consts.push(quote!(const #mname: #header = #formatted;));
            }
            (Some(name), None) => {
                let pname = if *bits == 1 {
                    format_ident!("FLAG_{name}")
                } else {
                    format_ident!("SLOT_{name}")
                };
                consts.push(quote!(const #pname: #header = #formatted;));
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

    Ok((quote!(#header), quote!(#(#consts)*)))
}
