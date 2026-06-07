use std::collections::HashSet;

use crate::wired::{
    r#struct::classify::{FieldKind, FieldShape, classify},
    utils::*,
};
use darling::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, *};

#[derive(Debug, FromMeta)]
pub struct Bounded {
    pub low: Expr,
    pub high: Expr,
}

#[derive(Debug, FromMeta)]
pub struct Scalar {
    #[darling(flatten)]
    pub store: ScalarStore,
    pub bounded: Option<Bounded>,
}

#[derive(Debug, FromMeta)]
pub enum ScalarStore {
    Format(Format),
    Slot(SlotIdent),
}

impl Default for ScalarStore {
    fn default() -> Self {
        Self::Format(Format::default())
    }
}

#[derive(Debug, FromMeta)]
pub struct Slice {
    pub len: Len,
    pub bounded: Option<Bounded>,
}

#[derive(Debug, Clone, Default, FromMeta)]
pub(crate) enum HeaderStore {
    #[default]
    Embedded,
    Flattened {
        shift: Expr,
    },
}

#[derive(Debug, Default, FromMeta)]
pub struct Msg {
    #[darling(default)]
    len: Len,
    #[darling(default)]
    header: HeaderStore,
}

#[derive(Debug, FromMeta)]
pub enum Attribute {
    Scalar(Scalar),
    Slice(Slice),
    Msg(Msg),
}

impl Default for Attribute {
    fn default() -> Self {
        Self::Msg(Msg::default())
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(wf))]
pub struct WiredField {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(flatten, default)]
    pub attr: Attribute,
    pub opt: Option<Optional>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum InnerField {
    Scalar { ty: Type, attr: Scalar },
    Slice { ty: Type, attr: Slice },
    Msg { ty: Type, sty: Type, attr: Msg },
}

#[rustfmt::skip]
impl InnerField {
    pub fn root(&self) -> TokenStream {
        match self {
            Self::Scalar { .. } => quote!(::elvwf::scalar),
            Self::Slice { .. } => quote!(::elvwf::slice),
            Self::Msg { .. } => quote!(::elvwf::msg),
        }
    }

    pub fn mode(&self) -> TokenStream {
        match self {
            Self::Scalar { .. } => quote!(),
            Self::Slice { attr: Slice { len, .. }, .. } => match len {
                Len::Prefixed(_) => quote!(),
                _ => quote!(::noprefix)
            },
            Self::Msg { attr: Msg { len, header }, .. } => match (len, header) {
                (Len::Prefixed(_), HeaderStore::Embedded) => quote!(),
                (Len::Prefixed(_), HeaderStore::Flattened { .. }) => quote!(::nohead),
                (_, HeaderStore::Embedded) => quote!(::noprefix),
                (_, HeaderStore::Flattened { .. }) => quote!(::noprefixnohead),
            },
        }
    }

    pub fn forward_header(&self) -> TokenStream {
        match self {
            Self::Msg { attr: Msg { header: HeaderStore::Flattened { shift }, .. }, .. } => quote!(, h >> #shift),
            _ => quote!(),
        }
    }

    pub fn generics(&self) -> TokenStream {
        let (sty, fmt) = match self {
            Self::Scalar { ty: sty, attr: Scalar { store: ScalarStore::Format(fmt), .. } }
            | Self::Slice { ty: sty, attr: Slice { len: Len::Prefixed(fmt), .. } }
            | Self::Msg { sty, attr: Msg { len: Len::Prefixed(fmt), .. }, .. } => (sty, Some(fmt)),

            Self::Scalar { ty: sty, .. } | Self::Slice { ty: sty, .. } | Self::Msg { sty, .. } => (sty, None),
        };

        match fmt {
            Some(fmt) => quote!(::<#sty, #fmt>),
            None => quote!(::<#sty>),
        }
    }

    pub fn user_length(&self) -> (TokenStream, TokenStream) {
        if let Self::Slice { ty: Type::Reference(TypeReference { elem, .. }), .. } = self {
            if let Type::Path(TypePath { qself: None, path}) = &**elem {
                if path.is_ident("str") {
                    return (quote!(.chars().count()), quote!(usize))
                } else if path.is_ident("CStr") {
                    return (quote!(.to_str().map(|s| s.chars().count()).map_err(::elvwf::slice::Error::from)?), quote!(usize))
                }
            } else {
                return (quote!(.len()), quote!(usize));
            }
        }

        let ty = match self {
            InnerField::Scalar { ty, .. } => ty,
            InnerField::Slice { ty, .. } => ty,
            InnerField::Msg { ty, .. } => ty,
        };

        (quote!(), quote!(#ty))
    }

    pub fn new(shape: FieldShape, attr: Attribute) -> syn::Result<Self> {
        let err = |ty: &Type, m| syn::Error::new(ty.span(), m);

        match shape {
            FieldShape::Scalar((ty, sty)) => match attr {
                Attribute::Scalar(attr) => Ok(Self::Scalar { ty: sty, attr }),
                _ => Err(err(&ty, "attribute `#[wf(scalar(..))]` must be set")),
            },
            FieldShape::Slice((ty, sty)) => match attr {
                Attribute::Slice(attr) => Ok(Self::Slice { ty: sty, attr }),
                _ => Err(err(&ty, "attribute `#[wf(slice(..))]` must be set")),
            },
            FieldShape::Msg((ty, sty)) => match attr {
                Attribute::Msg(attr) => Ok(Self::Msg { ty, sty, attr }),
                _ => Err(err(&ty, "attribute `#[wf(msg(..))]` must be set")),
            },
        }
    }
}

#[derive(Debug)]
pub enum Field {
    Optional {
        member: Member,
        inner: InnerField,
        flag: FlagIdent,
    },
    Conditional {
        member: Member,
        inner: InnerField,
        skip: Expr,
        flag: FlagIdent,
    },
    Required {
        member: Member,
        inner: InnerField,
    },
}

#[rustfmt::skip]
impl Field {
    pub fn inner(&self) -> &InnerField {
        let (Self::Optional { inner, .. }
        | Self::Conditional { inner, .. }
        | Self::Required { inner, .. }) = self;

        inner
    }

    pub fn mode(&self) -> TokenStream {
        match self {
            Self::Optional { .. } => quote!(::optional),
            Self::Conditional { .. } => quote!(::conditional),
            Self::Required { .. } => quote!(),
        }
    }

    pub fn root(&self) -> TokenStream {
        let inner = self.inner();
        append(inner.root(), self.mode(), inner.mode())
    }

    pub fn member(&self) -> &Member {
        let (Self::Optional { member, .. }
        | Self::Conditional { member, .. }
        | Self::Required { member, .. }) = self;

        member
    }

    pub fn access(&self) -> (TokenStream, TokenStream) {
        let (Self::Optional { member, inner, .. }
        | Self::Conditional { member, inner, .. }
        | Self::Required { member, inner, .. }) = self;

        match inner {
            InnerField::Msg { .. } => (quote!(self.#member), quote!(&self.#member)),
            _ => (quote!(self.#member), quote!(self.#member)),
        }
    }

    pub fn skip(&self) -> (TokenStream, TokenStream) {
        match self {
            Self::Conditional { skip, inner, .. } => match inner {
                InnerField::Msg { .. } => (quote!(, #skip), quote!(, &#skip)),
                _ => (quote!(, #skip), quote!(, #skip)),
            },
            _ => (quote!(), quote!()),
        }
    }

    pub fn length(&self) -> (TokenStream, TokenStream) {
        let (Self::Optional { member, inner, .. }
        | Self::Conditional { member, inner, .. }
        | Self::Required { member, inner, .. }) = self;

        let length = format_ident!("_{}_len", member);

        match inner {
            InnerField::Msg { attr: Msg { len, .. }, .. }
            | InnerField::Slice { attr: Slice { len, .. }, .. } => match len {
                Len::Slot(_) | Len::Embedded | Len::Remaining => {
                    (quote!(#length), quote!(, #length))
                }
                _ => (quote!(#length), quote!()),
            },
            _ => (quote!(#length), quote!()),
        }
    }

    pub fn flag(&self) -> (TokenStream, TokenStream) {
        let member = self.member();
        let flag = format_ident!("_{}_flag", member);

        (
            quote!(#flag),
            match self {
                Self::Conditional { .. } | Self::Optional { .. } => quote!(, #flag),
                _ => quote!(),
            },
        )
    }

    pub fn trigger(&self) -> Option<(TokenStream, &FlagIdent)> {
        match self {
            Self::Optional {
                member, flag: trigger, ..
            } => Some((quote!(self.#member.is_some()), trigger)),
            Self::Conditional {
                member,
                skip,
                flag: trigger,
                ..
            } => Some((quote!(self.#member != #skip), trigger)),
            _ => None,
        }
    }

    pub fn new(field: WiredField, index: usize) -> syn::Result<Self> {
        let member = match field.ident {
            Some(ident) => Member::Named(ident),
            None => Member::Unnamed(Index::from(index)),
        };

        let span = field.ty.span();

        match classify(&field.ty)? {
            FieldKind::Optional(shape) => {
                let inner = InnerField::new(shape, field.attr)?;
                match field.opt {
                    Some(Optional { condition: None, trigger }) => Ok(Self::Optional { member, inner, flag: trigger }),
                    _ => Err(syn::Error::new(span, "optional fields must have a `#[wf(opt(..))]` attribute with no condition")),
                }
            }
            FieldKind::Plain(shape) => {
                let inner = InnerField::new(shape, field.attr)?;
                match field.opt {
                    Some(Optional { condition: Some(cond), trigger }) => Ok(Self::Conditional { member, inner, skip: cond, flag: trigger }),
                    Some(Optional { condition: None, .. }) => Err(syn::Error::new(span, "`#[wf(opt(..))]` attribute must define a condition on non-option fields")),
                    None => Ok(Self::Required { member, inner }),
                }
            }
        }
    }
}

pub struct Inner {
    pub member: TokenStream,
    pub len: TokenStream,
    pub header: TokenStream,
    pub encode: TokenStream,
    pub decode: TokenStream,
    pub constraint: Option<TokenStream>,
    pub assert: Option<TokenStream>,
    pub mask: Option<TokenStream>,
}

#[rustfmt::skip]
pub fn inner(
    field: Field,
    lt: &Lifetime,
    hty: &TokenStream,
    last: bool,
    used_flags: &mut HashSet<FlagIdent>,
    used_slots: &mut HashSet<SlotIdent>
) -> syn::Result<Inner> {
    let member = field.member().clone();
    let id = format_ident!("_{}", member);

    let mode = field.mode();
    let root = field.root();
    let gnrcs = field.inner().generics();
    let h = field.inner().forward_header();
    let (user_len, len_ty) = field.inner().user_length();

    let (access, access_ref) = field.access();
    let (skip, skip_ref) = field.skip();
    let fallback = &skip;

    let (flag_dcl, flag) = field.flag();
    let (len_dcl, length) = field.length();

    let mut len = Vec::<TokenStream>::new();
    let mut header = Vec::<TokenStream>::new();
    let mut encode = Vec::<TokenStream>::new();
    let mut decode = Vec::<TokenStream>::new();
    let mut constraint = Option::<TokenStream>::None;
    let mut assert = Option::<TokenStream>::None;
    let mut mask = Option::<TokenStream>::None;

    if let Some((trigger, flag)) = field.trigger() {
        if !used_flags.insert(flag.clone()) {
            return Err(syn::Error::new(flag.span(), "flag already used."));
        }

        header.push(quote!(h = ::elvwf::header::trigger(h, #trigger, Self::#flag);));
        decode.push(quote!(let #flag_dcl = ::elvwf::header::has(h, Self::#flag);));
    }

    macro_rules! emit {
        () => {{
            len.push(quote!(len += #root ::len #gnrcs (#access_ref #skip_ref); ));
            encode.push(quote!(#root ::encode #gnrcs (buf, #access #skip)?;));
            decode.push(quote!(let #id = #root ::decode #gnrcs (buf #h #length #flag #fallback)?;));
        }};
    }

    if let InnerField::Msg { ty, attr: Msg { header: header_store, .. }, .. } = field.inner() {
        constraint.replace(quote!(#ty: ::elvwf::Wired<#lt>));

        if let HeaderStore::Flattened { shift } = header_store {
            let tty = to_turbofish(ty);

            constraint.replace(quote!(#ty: ::elvwf::Wired<#lt, Header = #hty>));
            assert.replace(quote! {
                assert!(mh & (#tty::WF_MASK_HEADER << #shift) == 0, "header overlap");
                mh = mh | (#tty::WF_MASK_HEADER << #shift);
            });
            mask.replace(quote!(#tty::WF_MASK_HEADER << #shift));

            header.push(quote!(h = ::elvwf::header::compose(h, ::elvwf::msg #mode ::header (#access_ref)? << #shift);));
        }
    }

    if let  InnerField::Scalar { attr: Scalar { store: ScalarStore::Slot(_), bounded: Some(Bounded { low, high }) }, .. }
            | InnerField::Slice { attr: Slice { len: Len::Slot(_), bounded: Some(Bounded { low, high }) }, .. } = field.inner() {
        let check = quote!(if !(#low..=#high as #len_ty).contains(& #access #user_len) { return Err(::elvwf::Error::FieldOutOfBound); });
        let check_id = quote!(if !(#low..=#high as #len_ty).contains(& #id #user_len) { return Err(::elvwf::Error::FieldOutOfBound); });
        let opt_check = quote!(if let Some(#id) = #access { #check_id });

        if matches!(field, Field::Optional { .. }) {
            header.push(opt_check);
        } else {
            header.push(check);
        }
    }

    if let  InnerField::Scalar { attr: Scalar { store: ScalarStore::Format(_), bounded: Some(Bounded { low, high }) }, .. }
            | InnerField::Slice { attr: Slice { len: Len::Prefixed(_) | Len::Remaining | Len::Embedded, bounded: Some(Bounded { low, high }) }, .. } = field.inner() {
        let check = quote!(if !(#low..=#high as #len_ty).contains(& #access #user_len) { return Err(::elvwf::Error::FieldOutOfBound); });
        let check_id = quote!(if !(#low..=#high as #len_ty).contains(& #id #user_len) { return Err(::elvwf::Error::FieldOutOfBound); });
        let opt_check = quote!(if let Some(#id) = #access { #check_id });

        if matches!(field, Field::Optional { .. }) {
            encode.push(opt_check);
        } else {
            encode.push(check);
        }
    }

    // First pass: exceptions that dont use 'emit'
    match field.inner() {
        InnerField::Scalar { attr: Scalar { store: ScalarStore::Slot(slot), .. }, .. } => {
            if !used_slots.insert(slot.clone()) {
                return Err(syn::Error::new(slot.span(), "slot already used."));
            }

            header.push(quote!(h = ::elvwf::header #mode ::put(h, #access, Self::#slot #skip)?;));
            decode.push(quote!(let #id = ::elvwf::header #mode ::get(h, Self::#slot #flag #fallback)?;));
        }
        rest => {
            // Second pass: exceptions that need extras
            if let InnerField::Slice { attr: Slice { len, .. }, .. } | InnerField::Msg { attr: Msg { len, .. }, .. } = rest
            {
                match len {
                    Len::Slot(slot) => {
                        if !used_slots.insert(slot.clone()) {
                            return Err(syn::Error::new(slot.span(), "slot already used."));
                        }

                        header.push(quote!(h = ::elvwf::header::put(h, #root ::len #gnrcs(#access_ref #skip_ref), Self::#slot)?;));
                        decode.push(quote!(let #len_dcl = ::elvwf::header::get(h, Self::#slot)?;));
                    }
                    Len::Embedded => {
                        decode.push(quote!(let #len_dcl: usize = 8;));
                    }
                    Len::Remaining => {
                        if !last {
                            return Err(syn::Error::new(member.span(), "attribute `len(remaining)` can only be put on the last field"));
                        }

                        decode.push(quote!(let #len_dcl: usize = len - (_bs - buf.len());));
                    }
                    _ => {}
                }
            }

            emit!();
        }
    }

    if let  InnerField::Scalar { attr: Scalar { bounded: Some(Bounded { low, high }), .. }, .. }
            | InnerField::Slice { attr: Slice { bounded: Some(Bounded { low, high }), .. }, .. } = field.inner() {
        let check = quote!(if !(#low..=#high as #len_ty).contains(& #id #user_len) { return Err(::elvwf::Error::FieldOutOfBound); });
        let opt_check = quote!(if let Some(#id) = #id { #check });

        if matches!(field, Field::Optional { .. }) {
            decode.push(opt_check);
        } else {
            decode.push(check);
        }
    }

    Ok(Inner {
        member: quote!(#member: #id),
        len: quote!(#(#len)*),
        header: quote!(#(#header)*),
        encode: quote!(#(#encode)*),
        decode: quote!(#(#decode)*),
        constraint,
        assert,
        mask,
    })
}
