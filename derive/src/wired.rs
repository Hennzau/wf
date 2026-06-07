use darling::*;
use proc_macro2::TokenStream;
use syn::*;

pub(crate) mod utils;

pub(crate) mod r#enum;
pub(crate) mod r#struct;
pub(crate) mod r#union;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wf), supports(struct_any))]
pub(crate) struct Struct {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ast::Data<darling::util::Ignored, r#struct::inner::WiredField>,

    #[darling(default, rename = "struct")]
    pub r#struct: r#struct::outer::StructParams,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wf), supports(enum_newtype))]
pub(crate) struct Union {
    pub ident: Ident,
    pub generics: Generics,
    pub data: ast::Data<r#union::inner::WiredVariant, darling::util::Ignored>,

    #[darling(rename = "union")]
    pub r#union: r#union::outer::UnionParams,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(wf), supports(enum_unit))]
pub(crate) struct Enum {
    pub ident: Ident,
    pub data: ast::Data<r#enum::inner::WiredVariant, darling::util::Ignored>,

    #[darling(rename = "enum")]
    pub r#enum: r#enum::outer::EnumParams,
}

#[derive(Debug)]
pub(crate) enum Msg {
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

impl FromDeriveInput for Msg {
    fn from_derive_input(input: &DeriveInput) -> darling::Result<Self> {
        match &input.data {
            syn::Data::Struct(_) => Struct::from_derive_input(input).map(Self::Struct),
            syn::Data::Enum(data) => {
                if data.variants.iter().any(|v| v.discriminant.is_some()) {
                    Enum::from_derive_input(input).map(Self::Enum)
                } else {
                    Union::from_derive_input(input).map(Self::Union)
                }
            }
            _ => Err(darling::Error::unsupported_shape("union")),
        }
    }
}

pub fn expand(input: Msg) -> syn::Result<TokenStream> {
    match input {
        Msg::Struct(input) => r#struct::expand(input),
        Msg::Union(input) => r#union::expand(input),
        Msg::Enum(input) => r#enum::expand(input),
    }
}
