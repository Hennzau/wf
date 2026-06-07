use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemMod, Path, TypePath};

#[derive(Debug, FromMeta)]
enum Tool {
    Optional,
    Conditional,
}

#[derive(Debug, FromMeta)]
struct DecodeParam {
    name: Ident,
    ty: TypePath,
}

#[derive(Debug, FromMeta)]
struct Config {
    tool: Tool,
    base_module: Path,
    template: TypePath,
    codec: Option<TypePath>,
    #[darling(default)]
    borrow: bool,
    #[darling(multiple, rename = "decode_param")]
    decode_params: Vec<DecodeParam>,
}

struct Fragment {
    declaration: TokenStream,
    usage: TokenStream,
}

impl Fragment {
    fn empty() -> Self {
        Self {
            declaration: quote!(),
            usage: quote!(),
        }
    }
}

fn codec_fragment(codec: &Option<TypePath>) -> Fragment {
    match codec {
        Some(codec) => Fragment {
            declaration: quote!(, C: #codec),
            usage: quote!(, C),
        },
        None => Fragment::empty(),
    }
}

fn decode_fragment(params: &[DecodeParam]) -> Fragment {
    let declaration = params
        .iter()
        .map(|DecodeParam { name, ty }| quote!(, #name: #ty))
        .collect();
    let usage = params
        .iter()
        .map(|DecodeParam { name, .. }| quote!(, #name))
        .collect();

    Fragment { declaration, usage }
}

pub fn expand(attrs: Vec<NestedMeta>, input: ItemMod) -> syn::Result<TokenStream> {
    let Config {
        tool,
        base_module,
        template,
        codec,
        borrow,
        decode_params,
    } = Config::from_list(&attrs)?;

    let module_ident = input.ident;
    let module_content = input.content.map(|(_, items)| items).unwrap_or_default();

    let codec = codec_fragment(&codec);
    let decode = decode_fragment(&decode_params);

    let codec_decl = &codec.declaration;
    let codec_use = &codec.usage;
    let decode_decl = &decode.declaration;
    let decode_use = &decode.usage;

    let by_ref = if borrow { quote!(&) } else { quote!() };

    let functions = match tool {
        Tool::Optional => quote! {
            pub fn len<'a, T: #template #codec_decl>(this: #by_ref Option<T>) -> usize {
                match this {
                    Some(this) => #base_module::len::<T #codec_use>(this),
                    None => 0,
                }
            }

            pub fn encode<'a, T: #template #codec_decl>(
                buf: &mut &mut [u8],
                this: Option<T>,
            ) -> Result<(), Error> {
                if let Some(this) = this {
                    #base_module::encode::<T #codec_use>(buf, this)?;
                }
                Ok(())
            }

            pub fn decode<'a, T: #template #codec_decl>(
                buf: &mut &'a [u8]
                #decode_decl,
                flag: bool,
            ) -> Result<Option<T>, Error> {
                if flag {
                    #base_module::decode::<T #codec_use>(buf #decode_use).map(Some)
                } else {
                    Ok(None)
                }
            }
        },
        Tool::Conditional => quote! {
            pub fn len<'a, T: #template + PartialEq #codec_decl>(
                this: #by_ref T,
                skip: #by_ref T,
            ) -> usize {
                if this == skip {
                    return 0;
                }
                #base_module::len::<T #codec_use>(this)
            }

            pub fn encode<'a, T: #template + PartialEq #codec_decl>(
                buf: &mut &mut [u8],
                this: T,
                skip: T,
            ) -> Result<(), Error> {
                if this == skip {
                    return Ok(());
                }
                #base_module::encode::<T #codec_use>(buf, this)
            }

            pub fn decode<'a, T: #template #codec_decl>(
                buf: &mut &'a [u8] #decode_decl,
                flag: bool,
                fallback: T,
            ) -> Result<T, Error> {
                if flag {
                    #base_module::decode::<T #codec_use>(buf #decode_use)
                } else {
                    Ok(fallback)
                }
            }
        },
    };

    Ok(quote! {
        pub mod #module_ident {
            #(#module_content)*

            #functions
        }
    })
}
