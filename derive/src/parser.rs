use crate::{
    LenSpec, OptSpec, TriggerSource, WiredField,
    classify::{FieldKind, FieldShape},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, *};

pub struct ParsedField<'a> {
    pub ident: &'a Ident,
    pub len: TokenStream,
    pub header: TokenStream,
    pub encode: TokenStream,
    pub decode: TokenStream,
}

macro_rules! required {
    ($field: ident, $span: expr) => {
        $field.ok_or_else(|| {
            Error::new(
                $span,
                "FATAL ERROR: this field has not been recognized, this is strange, report it",
            )
        })?
    };

    ($field: ident, $expr: expr, $span: expr) => {
        if $field.replace($expr).is_some() {
            return Err(Error::new(
                $span,
                "FATAL ERROR: this field has been filled multiple times, this is strange, report it",
            ));
        }
    };
}

impl<'a> ParsedField<'a> {
    pub fn new(field: &'a WiredField, kind: FieldKind) -> Result<Self> {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new(field.ty.span(), "Wired requires named fields"))?;

        let mut len = None;
        let mut header = Vec::new();
        let mut encode = None;
        let mut decode = None;

        let mut extra = Vec::new();

        let ident_opt = format_ident!("{}_opt", ident);
        let ident_len = format_ident!("{}_len", ident);

        match &kind {
            FieldKind::Optional(ty) => {
                let trigger = field.opt_no_condition()?;

                match trigger {
                    TriggerSource::Flag(flag) => {
                        let flag = format_ident!("FLAG_{}", flag);

                        header.push(quote!(h = elvwf::header::trigger(h, self.#ident.is_some(), Self::#flag);));
                        extra.push(quote!(let #ident_opt = elvwf::header::has(h, Self::#flag);));
                    }
                }

                match ty {
                    FieldShape::Scalar(ty) => {
                        field.no_len()?;
                        let format = field.format()?;

                        required!(
                            len,
                            quote!(len += elvwf::scalar::opt::len::<#ty, elvwf::scalar::#format>(self.#ident);),
                            ident.span()
                        );

                        required!(
                            encode,
                            quote!(elvwf::scalar::opt::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                            ident.span()
                        );

                        required!(
                            decode,
                            quote! {
                                #(#extra)*
                                let #ident = elvwf::scalar::opt::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt)?;
                            },
                            ident.span()
                        );
                    }
                    FieldShape::Slice(ty) => {
                        field.no_format()?;

                        match field.len()? {
                            LenSpec::Prefixed { format } => {
                                required!(
                                    len,
                                    quote!(len += elvwf::slice::opt::prefixed_len::<#ty, elvwf::scalar::#format>(self.#ident);),
                                    ident.span()
                                );

                                required!(
                                    encode,
                                    quote!(elvwf::slice::opt::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                                    ident.span()
                                );

                                required!(
                                    decode,
                                    quote! {
                                        #(#extra)*
                                        let #ident = elvwf::slice::opt::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt)?;
                                    },
                                    ident.span()
                                );
                            }
                            LenSpec::Slot(slot) => {
                                let slot = format_ident!("SLOT_{}", slot);
                                header.push(quote!(h = elvwf::header::put(h, elvwf::slice::opt::value_len::<#ty>(self.#ident), Self::#slot)?;));
                                extra.push(
                                    quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                );

                                required!(
                                    len,
                                    quote!(len += elvwf::slice::opt::value_len::<#ty>(self.#ident);),
                                    ident.span()
                                );

                                required!(
                                    encode,
                                    quote!(elvwf::slice::opt::encode_value::<#ty>(buf, self.#ident)?;),
                                    ident.span()
                                );

                                required!(
                                    decode,
                                    quote! {
                                        #(#extra)*
                                        let #ident = elvwf::slice::opt::decode_value::<#ty>(buf, #ident_opt, #ident_len)?;
                                    },
                                    ident.span()
                                );
                            }
                        }
                    }
                    FieldShape::Msg(ty) => {
                        field.no_format()?;

                        match field.len()? {
                            LenSpec::Prefixed { format } => {
                                required!(
                                    len,
                                    quote!(len += elvwf::msg::opt::prefixed_len::<#ty, elvwf::scalar::#format>(&self.#ident);),
                                    ident.span()
                                );

                                required!(
                                    encode,
                                    quote!(elvwf::msg::opt::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                                    ident.span()
                                );

                                required!(
                                    decode,
                                    quote! {
                                        #(#extra)*
                                        let #ident = elvwf::msg::opt::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt)?;
                                    },
                                    ident.span()
                                );
                            }
                            LenSpec::Slot(slot) => {
                                let slot = format_ident!("SLOT_{}", slot);
                                header.push(quote!(h = elvwf::header::put(h, elvwf::slice::msg::value_len::<#ty>(self.#ident), Self::#slot)?;));
                                extra.push(
                                    quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                );

                                required!(
                                    len,
                                    quote!(len += elvwf::msg::opt::value_len::<#ty>(&self.#ident);),
                                    ident.span()
                                );

                                required!(
                                    encode,
                                    quote!(elvwf::msg::opt::encode_value::<#ty>(buf, self.#ident)?;),
                                    ident.span()
                                );

                                required!(
                                    decode,
                                    quote! {
                                        #(#extra)*
                                        let #ident = elvwf::msg::opt::decode_value::<#ty>(buf, #ident_opt, #ident_len)?;
                                    },
                                    ident.span()
                                );
                            }
                        }
                    }
                }
            }
            FieldKind::Required(ty) => {
                if field.opt.is_none() {
                    match ty {
                        FieldShape::Scalar(ty) => {
                            field.no_len()?;
                            let format = field.format()?;

                            required!(
                                len,
                                quote!(len += elvwf::scalar::len::<#ty, elvwf::scalar::#format>(self.#ident);),
                                ident.span()
                            );

                            required!(
                                encode,
                                quote!(elvwf::scalar::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                                ident.span()
                            );

                            required!(
                                decode,
                                quote! {
                                    let #ident = elvwf::scalar::decode::<#ty, elvwf::scalar::#format>(buf)?;
                                },
                                ident.span()
                            );
                        }
                        FieldShape::Slice(ty) => {
                            field.no_format()?;

                            match field.len()? {
                                LenSpec::Prefixed { format } => {
                                    required!(
                                        len,
                                        quote!(len += elvwf::slice::prefixed_len::<#ty, elvwf::scalar::#format>(self.#ident);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::slice::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            let #ident = elvwf::slice::decode::<#ty, elvwf::scalar::#format>(buf)?;
                                        },
                                        ident.span()
                                    );
                                }
                                LenSpec::Slot(slot) => {
                                    let slot = format_ident!("SLOT_{}", slot);
                                    header.push(quote!(h = elvwf::header::put(h, elvwf::slice::value_len::<#ty>(self.#ident), Self::#slot)?;));
                                    extra.push(
                                        quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                    );

                                    required!(
                                        len,
                                        quote!(len += elvwf::slice::value_len::<#ty>(self.#ident);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::slice::encode_value::<#ty>(buf, self.#ident)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::slice::decode_value::<#ty>(buf, #ident_len)?;
                                        },
                                        ident.span()
                                    );
                                }
                            }
                        }
                        FieldShape::Msg(ty) => {
                            field.no_format()?;

                            match field.len()? {
                                LenSpec::Prefixed { format } => {
                                    required!(
                                        len,
                                        quote!(len += elvwf::msg::prefixed_len::<#ty, elvwf::scalar::#format>(&self.#ident);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::msg::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            let #ident = elvwf::msg::decode::<#ty, elvwf::scalar::#format>(buf)?;
                                        },
                                        ident.span()
                                    );
                                }
                                LenSpec::Slot(slot) => {
                                    let slot = format_ident!("SLOT_{}", slot);
                                    header.push(quote!(h = elvwf::header::put(h, elvwf::msg::value_len::<#ty>(self.#ident), Self::#slot)?;));
                                    extra.push(
                                        quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                    );

                                    required!(
                                        len,
                                        quote!(len += elvwf::msg::value_len::<#ty>(&self.#ident);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::msg::encode_value::<#ty>(buf, self.#ident)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::msg::decode_value::<#ty>(buf, #ident_len)?;
                                        },
                                        ident.span()
                                    );
                                }
                            }
                        }
                    }
                } else {
                    let (condition, trigger) = field.opt_condition()?;

                    match trigger {
                        TriggerSource::Flag(flag) => {
                            let flag = format_ident!("FLAG_{}", flag);

                            header.push(quote!(h = elvwf::header::trigger(h, self.#ident == #condition, Self::#flag);));
                            extra
                                .push(quote!(let #ident_opt = elvwf::header::has(h, Self::#flag);));
                        }
                    }

                    match ty {
                        FieldShape::Scalar(ty) => {
                            field.no_len()?;
                            let format = field.format()?;

                            required!(
                                len,
                                quote!(len += elvwf::scalar::cond::len::<#ty, elvwf::scalar::#format>(self.#ident, #condition);),
                                ident.span()
                            );

                            required!(
                                encode,
                                quote!(elvwf::scalar::cond::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident, #condition)?;),
                                ident.span()
                            );

                            required!(
                                decode,
                                quote! {
                                    #(#extra)*
                                    let #ident = elvwf::scalar::cond::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt, #condition)?;
                                },
                                ident.span()
                            );
                        }
                        FieldShape::Slice(ty) => {
                            field.no_format()?;

                            match field.len()? {
                                LenSpec::Prefixed { format } => {
                                    required!(
                                        len,
                                        quote!(len += elvwf::slice::cond::prefixed_len::<#ty, elvwf::scalar::#format>(self.#ident, #condition);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::slice::cond::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident, #condition)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::slice::cond::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt, #condition)?;
                                        },
                                        ident.span()
                                    );
                                }
                                LenSpec::Slot(slot) => {
                                    let slot = format_ident!("SLOT_{}", slot);
                                    header.push(quote!(h = elvwf::header::put(h, elvwf::slice::cond::value_len::<#ty>(self.#ident, #condition), Self::#slot)?;));
                                    extra.push(
                                        quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                    );

                                    required!(
                                        len,
                                        quote!(len += elvwf::slice::cond::value_len::<#ty>(self.#ident, #condition);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::slice::cond::encode_value::<#ty>(buf, self.#ident, #condition)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::slice::cond::decode_value::<#ty>(buf, #ident_opt, #condition, #ident_len)?;
                                        },
                                        ident.span()
                                    );
                                }
                            }
                        }
                        FieldShape::Msg(ty) => {
                            field.no_format()?;

                            match field.len()? {
                                LenSpec::Prefixed { format } => {
                                    required!(
                                        len,
                                        quote!(len += elvwf::msg::cond::prefixed_len::<#ty, elvwf::scalar::#format>(&self.#ident, &#condition);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::msg::cond::encode::<#ty, elvwf::scalar::#format>(buf, self.#ident, #condition)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::msg::cond::decode::<#ty, elvwf::scalar::#format>(buf, #ident_opt, #condition)?;
                                        },
                                        ident.span()
                                    );
                                }
                                LenSpec::Slot(slot) => {
                                    let slot = format_ident!("SLOT_{}", slot);
                                    header.push(quote!(h = elvwf::header::put(h, elvwf::msg::cond::value_len::<#ty>(self.#ident, #condition), Self::#slot)?;));
                                    extra.push(
                                        quote!(let #ident_len = elvwf::header::get(h, Self::#slot)?;),
                                    );

                                    required!(
                                        len,
                                        quote!(len += elvwf::msg::cond::value_len::<#ty>(&self.#ident, &#condition);),
                                        ident.span()
                                    );

                                    required!(
                                        encode,
                                        quote!(elvwf::msg::cond::encode_value::<#ty>(buf, self.#ident, #condition)?;),
                                        ident.span()
                                    );

                                    required!(
                                        decode,
                                        quote! {
                                            #(#extra)*
                                            let #ident = elvwf::msg::cond::decode_value::<#ty>(buf, #ident_opt, #condition, #ident_len)?;
                                        },
                                        ident.span()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            ident,
            len: required!(len, ident.span()),
            header: quote!(#(#header)*),
            encode: required!(encode, ident.span()),
            decode: required!(decode, ident.span()),
        })
    }
}

impl WiredField {
    pub fn no_len(&self) -> Result<()> {
        if self.len.is_some() {
            return Err(Error::new(
                self.ty.span(),
                "This type must not have a `len` attribute",
            ));
        }

        Ok(())
    }

    pub fn no_format(&self) -> Result<()> {
        if self.format.is_some() {
            return Err(Error::new(
                self.ty.span(),
                "This type must not have a `format` attribute",
            ));
        }

        Ok(())
    }

    pub fn opt_condition(&self) -> Result<(&Expr, &TriggerSource)> {
        self.opt
            .as_ref()
            .ok_or_else(|| Error::new(self.ty.span(), "This type requires an `opt` attribute"))
            .map(|opt| {
                let OptSpec {
                    condition: Some(condition),
                    trigger,
                } = opt
                else {
                    return Err(Error::new(
                        self.ty.span(),
                        "This type must not have an `opt(if = ..)` attribute",
                    ));
                };

                Ok((condition, trigger))
            })
            .flatten()
    }

    pub fn opt_no_condition(&self) -> Result<&TriggerSource> {
        self.opt
            .as_ref()
            .ok_or_else(|| Error::new(self.ty.span(), "This type requires an `opt` attribute"))
            .map(|opt| {
                if opt.condition.is_none() {
                    Ok::<_, Error>(&opt.trigger)
                } else {
                    Err(Error::new(
                        self.ty.span(),
                        "This type must not have an `opt(if = ..)` attribute",
                    ))
                }
            })
            .flatten()
    }

    pub fn len(&self) -> Result<&LenSpec> {
        self.len
            .as_ref()
            .ok_or_else(|| Error::new(self.ty.span(), "This type requires a `len` attribute"))
    }

    pub fn format(&self) -> Result<&Ident> {
        self.format
            .as_ref()
            .ok_or_else(|| Error::new(self.ty.span(), "This type requires a `format` attribute"))
    }
}
