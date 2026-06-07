use std::collections::{HashMap, HashSet};

use darling::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, *};

#[derive(Debug, FromAttributes)]
#[darling(attributes(wfe))]
pub struct EnumError {
    domain: Ident,
    #[darling(default)]
    extends: HashMap<Ident, ()>,
}

#[derive(Debug, FromVariant, Clone)]
#[darling(attributes(wfe))]
pub struct EnumVariant {
    ident: Ident,
    error: LitStr,
    discriminant: Option<Expr>,
}

struct Error {
    ident: Ident,
    all_extended: Vec<Ident>,
    all_variants: Vec<EnumVariant>,
}

pub fn expand(input: ItemMod) -> syn::Result<TokenStream> {
    let (_, items) = input
        .content
        .as_ref()
        .ok_or_else(|| syn::Error::new(input.span(), "module should contain a body"))?;

    let errors = collect_errors(items)?;

    let mut ts = Vec::with_capacity(errors.len());
    for error in errors.values() {
        ts.push(generate_enum(error, &errors)?);
    }

    Ok(quote!(#(#ts)*))
}

fn collect_errors(items: &[Item]) -> syn::Result<HashMap<Ident, Error>> {
    struct ErrorP1 {
        ident: Ident,
        extends: Vec<Ident>,
        variants: Vec<EnumVariant>,
    }

    let mut errors_p1 = HashMap::<Ident, ErrorP1>::new();

    for item in items {
        let Item::Enum(item) = item else {
            return Err(syn::Error::new(
                item.span(),
                "module must only contain enums",
            ));
        };

        let EnumError { domain, extends } = EnumError::from_attributes(&item.attrs)?;

        let error = errors_p1.entry(domain).or_insert_with(|| ErrorP1 {
            ident: item.ident.clone(),
            extends: extends.keys().cloned().collect(),
            variants: Vec::new(),
        });

        for variant in &item.variants {
            error.variants.push(EnumVariant::from_variant(variant)?);
        }
    }

    let mut errors = HashMap::<Ident, Error>::new();

    for (domain, error) in &errors_p1 {
        let mut all_variants: Vec<&EnumVariant> = error.variants.iter().collect();
        let mut visited = HashSet::<&Ident>::new();
        let mut to_visit: Vec<&Ident> = error.extends.iter().collect();

        while let Some(domain) = to_visit.pop() {
            if !visited.insert(domain) {
                continue;
            }

            let Some(extended) = errors_p1.get(domain) else {
                return Err(syn::Error::new(
                    error.ident.span(),
                    "extending an unknown domain",
                ));
            };

            all_variants.extend(&extended.variants);

            for domain in &extended.extends {
                if !visited.contains(domain) {
                    to_visit.push(domain);
                }
            }
        }

        let all_variants = Vec::from_iter(all_variants.into_iter().cloned());

        errors.insert(
            domain.clone(),
            Error {
                ident: error.ident.clone(),
                all_extended: visited.into_iter().cloned().collect(),
                all_variants,
            },
        );
    }

    Ok(errors)
}

fn generate_enum(error: &Error, errors: &HashMap<Ident, Error>) -> syn::Result<TokenStream> {
    let ident = &error.ident;
    let all_variants = &error.all_variants;

    let mut vts = Vec::with_capacity(all_variants.len());
    let mut dts = Vec::with_capacity(all_variants.len());
    let mut fts = Vec::new();

    for domain in &error.all_extended {
        let extended = errors.get(domain).unwrap();
        let eid = &extended.ident;

        let fvts = extended.all_variants.iter().map(|variant| {
            let vid = &variant.ident;
            quote!(#eid::#vid => Self::#vid)
        });

        fts.push(quote!(
            impl From<#eid> for #ident {
                fn from(value: #eid) -> Self {
                    match value {
                        #(#fvts),*
                    }
                }
            }
        ));
    }

    for variant in all_variants {
        let EnumVariant {
            ident,
            error,
            discriminant,
        } = variant;

        let Some(Expr::Lit(ExprLit {
            lit: Lit::Int(discriminant),
            ..
        })) = discriminant
        else {
            return Err(syn::Error::new(
                ident.span(),
                "variant must have an int discriminant",
            ));
        };

        vts.push(quote!(#ident = #discriminant));
        dts.push(quote!(Self::#ident => #error));
    }

    Ok(quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[repr(u8)]
        pub enum #ident {
            #(#vts),*
        }

        impl ::core::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(match self {
                    #(#dts),*
                })
            }
        }

        impl ::core::error::Error for #ident {}

        #(#fts)*
    })
}
