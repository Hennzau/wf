use proc_macro2::TokenStream;

use crate::wired::Msg;

pub(crate) mod r#enum;
pub(crate) mod r#struct;
pub(crate) mod r#union;

pub fn expand(input: Msg) -> syn::Result<TokenStream> {
    match input {
        Msg::Struct(input) => r#struct::expand(input),
        Msg::Union(input) => r#union::expand(input),
        Msg::Enum(input) => r#enum::expand(input),
    }
}
