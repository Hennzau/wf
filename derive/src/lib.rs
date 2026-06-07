use darling::{ast::NestedMeta, *};
use quote::quote;
use syn::*;

pub(crate) mod error;
#[cfg(feature = "randomized")]
pub(crate) mod randomized;
pub(crate) mod utils;
pub(crate) mod wired;

#[proc_macro_attribute]
pub fn wfe(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemMod);

    match error::expand(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn wfu(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemMod);
    let attrs = match NestedMeta::parse_meta_list(attrs.into()) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    match utils::expand(attrs, input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Wired, attributes(wf))]
pub fn derive_wired(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let parsed = match wired::Msg::from_derive_input(&input) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    match wired::expand(parsed) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(WiredBlank, attributes(wf))]
pub fn derive_wired_blank(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote!().into()
}

#[cfg(feature = "randomized")]
#[proc_macro_derive(Randomized, attributes(wf))]
pub fn derive_randomized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let parsed = match wired::Msg::from_derive_input(&input) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    match randomized::expand(parsed) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
