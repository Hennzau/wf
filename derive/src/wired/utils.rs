use proc_macro2::*;
use syn::{visit_mut::*, *};

use darling::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn to_turbofish(ty: &Type) -> Type {
    let mut ty = ty.clone();

    if let Type::Path(ref mut type_path) = ty {
        turbofish_path(&mut type_path.path);
    }

    ty
}

fn turbofish_path(path: &mut Path) {
    for segment in &mut path.segments {
        if let PathArguments::AngleBracketed(ref mut args) = segment.arguments {
            args.colon2_token = Some(token::PathSep::default());
        }
    }
}

pub fn append(a: TokenStream, b: TokenStream, c: TokenStream) -> TokenStream {
    quote!(#a #b #c)
}

#[derive(Debug, Default, FromMeta)]
pub(crate) enum Format {
    Le,
    Be,
    #[default]
    Ne,
    Vle,
}

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Self::Le => quote!(::elvwf::Le),
            Self::Be => quote!(::elvwf::Be),
            Self::Ne => quote!(::elvwf::Ne),
            Self::Vle => quote!(::elvwf::VLE),
        };
        tokens.extend(ts);
    }
}

#[derive(Debug, Default, FromMeta)]
pub(crate) enum FixedFormat {
    Le,
    Be,
    #[default]
    Ne,
}

impl ToTokens for FixedFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self {
            Self::Le => quote!(::elvwf::Le),
            Self::Be => quote!(::elvwf::Be),
            Self::Ne => quote!(::elvwf::Ne),
        };
        tokens.extend(ts);
    }
}

#[derive(Debug, Clone, FromMeta, Hash, PartialEq, Eq)]
pub struct SlotIdent(Ident);

impl ToTokens for SlotIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = format_ident!("WF_SLOT_{}", self.0);
        tokens.extend(quote!(#ident));
    }
}

#[derive(Debug, FromMeta)]
pub struct MaskIdent(pub Ident);

impl ToTokens for MaskIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = format_ident!("WF_MASK_{}", self.0);
        tokens.extend(quote!(#ident));
    }
}

#[derive(Debug, Default, FromMeta)]
pub(crate) enum Len {
    Prefixed(Format),
    Slot(SlotIdent),
    Remaining,
    #[default]
    Embedded,
}

#[derive(Debug, Clone, FromMeta, Hash, PartialEq, Eq)]
pub struct FlagIdent(Ident);

impl ToTokens for FlagIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = format_ident!("WF_FLAG_{}", self.0);
        tokens.extend(quote!(#ident));
    }
}

#[derive(Debug, FromMeta)]
pub(crate) struct Optional {
    #[darling(rename = "if")]
    pub condition: Option<Expr>,
    pub trigger: FlagIdent,
}

pub fn strip_lifetimes(ty: &Type) -> Type {
    let mut ty = ty.clone();
    StripLifetimes.visit_type_mut(&mut ty);
    ty
}

struct StripLifetimes;

impl VisitMut for StripLifetimes {
    fn visit_type_reference_mut(&mut self, node: &mut TypeReference) {
        node.lifetime = None;
        syn::visit_mut::visit_type_reference_mut(self, node);
    }

    fn visit_path_arguments_mut(&mut self, node: &mut PathArguments) {
        if let PathArguments::AngleBracketed(args) = node {
            let filtered: syn::punctuated::Punctuated<GenericArgument, Token![,]> = args
                .args
                .iter()
                .filter(|arg| !matches!(arg, GenericArgument::Lifetime(_)))
                .cloned()
                .collect();

            if filtered.is_empty() {
                *node = PathArguments::None;
                return;
            } else {
                args.args = filtered;
            }
        }
        syn::visit_mut::visit_path_arguments_mut(self, node);
    }
}
