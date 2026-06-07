use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

use crate::wired::{
    Struct,
    r#struct::inner::{Bounded, Field, InnerField, Scalar, Slice},
};

pub fn expand(input: Struct) -> syn::Result<TokenStream> {
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
        .take_struct()
        .expect("supports(struct_*) guarantees a struct");

    let mut members = Vec::<TokenStream>::with_capacity(fields.len());
    let mut constraints = Vec::<TokenStream>::with_capacity(fields.len());
    let mut random = Vec::<TokenStream>::with_capacity(fields.len());

    for (i, field) in fields.into_iter().enumerate() {
        let field = Field::new(field, i)?;

        let member = field.member();
        let id = format_ident!("_{}", member);
        members.push(quote!(#member: #id));

        let mode = field.mode();
        let (skip, _) = field.skip();

        match field.inner() {
            InnerField::Scalar {
                ty,
                attr: Scalar { bounded, .. },
            } => {
                let (low, high) = if let Some(Bounded { low, high }) = bounded {
                    (quote!(#low), quote!(#high))
                } else {
                    (quote!(#ty::MIN), quote!(#ty::MAX))
                };

                random.push(
                    quote!(let #id = ::elvwf::scalar #mode ::randomized::<#ty>(#low, #high #skip);),
                );
            }
            InnerField::Slice {
                ty,
                attr: Slice { bounded, .. },
            } => {
                let (low, high) = if let Some(Bounded { low, high }) = bounded {
                    (quote!(#low), quote!(#high))
                } else {
                    (quote!(usize::MIN), quote!(usize::MAX))
                };

                random.push(
                    quote!(let #id = ::elvwf::slice #mode ::randomized::<#ty>(buf, #low, #high #skip)?;),
                );
            }
            InnerField::Msg { ty, sty, .. } => {
                constraints.push(quote!(#ty: ::elvwf::Randomized<#lt>));
                random.push(quote!(let #id = ::elvwf::msg #mode ::randomized::<#sty>(buf #skip)?;));
            }
        }
    }

    let where_clause: WhereClause = where_clause
        .cloned()
        .map(|w| parse_quote!(#w #(#constraints),*))
        .unwrap_or(parse_quote!(where #(#constraints),*));

    Ok(quote!(
        #[allow(unused_braces)]
        impl #impl_generics ::elvwf::Randomized<#lt> for #ident #ty_generics #where_clause {
            fn randomized(buf: &mut &'a mut [u8]) -> Result<Self, ::elvwf::Error>
            where
                Self: Sized,
            {
                #(#random)*
                Ok(Self { #(#members),* })
            }
        }
    ))
}
