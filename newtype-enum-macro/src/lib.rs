#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::filter_map, clippy::default_trait_access)]

//! Provide the `#[newtype_enum]` attribute macro to derive the `Enum` and `Variant` traits from the `newtype-enum` crate.

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, parse_quote, parse_str, Error, Fields, ItemEnum, ItemStruct, LitStr,
    Meta, MetaList, NestedMeta, Path, Variant, VisRestricted, Visibility,
};

/// Derive the `Enum` and `Variant` traits from the `newtype-enum` crate.
#[proc_macro_attribute]
pub fn newtype_enum(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr: TokenStream = attr.into();
    let attr: proc_macro::TokenStream = quote!(_(#attr)).into();
    newtype_enum_impl(parse_macro_input!(attr), parse_macro_input!(item)).into()
}

macro_rules! unwrap_or_compile_error {
    ($expr:expr) => {
        match $expr {
            Ok(vis) => vis,
            Err(err) => return err.to_compile_error(),
        };
    };
}

fn newtype_enum_impl(meta: MetaList, item: ItemEnum) -> TokenStream {
    let e = unwrap_or_compile_error!(NewtypeEnum::new(meta, item));

    let enum_item = e.define_enum();
    let mod_variants = e.define_variants();
    let impl_variants = e.implement_variants();
    quote! {
        #enum_item
        #mod_variants
        #impl_variants
    }
}

struct NewtypeEnum {
    item: ItemEnum,
    crate_name: Path,
    variants: Ident,
    variants_vis: Visibility,
}

impl NewtypeEnum {
    fn new(meta: MetaList, item: ItemEnum) -> Result<Self, Error> {
        let crate_name = crate_name();
        let crate_name = parse_quote!(::#crate_name);

        let variants = ident_append(&item.ident, "_variants");
        let variants_vis = item.vis.clone();
        let mut e = Self {
            item,
            crate_name,
            variants,
            variants_vis,
        };

        for meta in meta.nested {
            match meta {
                NestedMeta::Meta(Meta::NameValue(meta)) => {
                    let ident: Ident = parse2(meta.path.to_token_stream())?;
                    match ident.to_string().as_ref() {
                        "variants" => {
                            let lit: LitStr = parse2(meta.lit.to_token_stream())?;
                            let lit = &lit;
                            let s = lit.value();
                            let s = if let Some(index) = s.rfind(' ') {
                                e.variants_vis = parse_str(&s[..index])
                                    .map_err(|err| Error::new_spanned(lit, err))?;
                                &s[index + 1..]
                            } else {
                                &s
                            };
                            e.variants =
                                parse_str(s).map_err(|err| Error::new_spanned(lit, err))?;
                        }
                        "unstable_self_test" => e.crate_name = parse_quote!(crate),
                        _ => return Err(Error::new_spanned(ident, "unknown argument")),
                    }
                }
                _ => return Err(Error::new_spanned(meta, "expected a name-value pair")),
            }
        }

        Ok(e)
    }

    fn variants(&self) -> impl Iterator<Item = Variant> + '_ {
        self.item.variants.iter().map(move |var| match &var.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let mut var = var.clone();
                var.discriminant = None;
                var
            }
            _ => {
                let ident = &var.ident;
                let variants = &self.variants;
                let doc = format!("See [`{0}`]({1}/struct.{0}.html).", ident, variants);
                parse_quote! {
                    #[doc = #doc]
                    #ident(#variants::#ident)
                }
            }
        })
    }

    fn define_enum(&self) -> ItemEnum {
        let mut item = self.item.clone();
        item.variants = self.variants().collect();
        item
    }

    fn define_variants(&self) -> TokenStream {
        let vis = unwrap_or_compile_error!(super_vis(&self.item.vis, || parse_quote!(pub(super))));
        let items: Vec<_> = self
            .item
            .variants
            .iter()
            .filter(|var| match &var.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => false,
                _ => true,
            })
            .map(move |var| {
                let mut item = ItemStruct {
                    attrs: self
                        .item
                        .attrs
                        .iter()
                        .filter(|attr| match attr.parse_meta() {
                            Ok(Meta::NameValue(meta)) if meta.path.is_ident("doc") => false,
                            _ => true,
                        })
                        .chain(var.attrs.iter())
                        .cloned()
                        .collect(),
                    vis: vis.clone(),
                    struct_token: Default::default(),
                    ident: var.ident.clone(),
                    generics: Default::default(),
                    fields: var.fields.clone(),
                    semi_token: None,
                };
                match &mut item.fields {
                    Fields::Unit => {
                        item.semi_token = parse_quote!(;);
                    }
                    Fields::Named(fields) => {
                        for field in &mut fields.named {
                            field.vis =
                                unwrap_or_compile_error!(super_vis(&field.vis, || vis.clone()));
                        }
                    }
                    _ => {
                        return Error::new_spanned(var, "unsupported variant type")
                            .to_compile_error();
                    }
                };
                quote!(#item)
            })
            .collect();
        if items.is_empty() {
            return Default::default();
        }
        let variants = &self.variants;
        let variants_vis = &self.variants_vis;
        let doc = format!("The generated variants of the `{0}` enum.", self.item.ident);
        quote! {
            #[allow(non_snake_case)]
            #[doc = #doc]
            #variants_vis mod #variants {
                use super::*;
                #(#items)*
            }
        }
    }

    fn implement_variants(&self) -> TokenStream {
        let e = &self.item.ident;
        let crate_name = &self.crate_name;
        let impls = self.variants().map(|var| {
            let ident = &var.ident;
            let ty = &var.fields.iter().next().unwrap().ty;
            let v = quote!(#e::#ident);
            let match_from = quote!{
                match e {
                    #v(v) => Some(v),
                    _ => None,
                }
            };
            quote!(
                #[doc(hidden)]
                impl #crate_name::unstable::VariantCore<#e> for #ty {
                     fn into_enum(self) -> #e {
                         #v(self)
                     }

                    fn from_enum(e: #e) -> ::core::option::Option<Self> {
                        #match_from
                    }

                    fn ref_enum(e: &#e) -> ::core::option::Option<&Self>{
                        #match_from
                    }

                    fn mut_enum(e: &mut #e) -> ::core::option::Option<&mut Self> {
                        #match_from
                    }

                    fn is_enum_variant(e: &#e) -> bool
                    {
                        match e {
                            #v(_) => true,
                            _ => false,
                        }
                    }

                    fn from_enum_unwrap(e: #e) -> Self {
                        match e {
                            #v(v) => v,
                            _ => ::core::panic!("called `Variant::from_enum_unwrap` on another enum variant"),
                        }
                    }

                    unsafe fn from_enum_unchecked(e: #e) -> Self {
                        match e {
                            #v(v) => v,
                            _ => ::core::hint::unreachable_unchecked(),
                        }
                    }
                }
                impl #crate_name::Variant<#e> for #ty { }
            )
        });
        quote! {
            const _: () = {
                impl #crate_name::Enum for #e { }
                #(#impls)*
            };
        }
    }
}

fn super_vis(vis: &Visibility, default: impl FnOnce() -> Visibility) -> Result<Visibility, Error> {
    let vis = match vis {
        Visibility::Inherited => default(),
        Visibility::Restricted(VisRestricted { in_token, path, .. }) => {
            if let Some(in_token) = in_token {
                if path.leading_colon.is_some() {
                    vis.clone()
                } else {
                    parse_quote!( pub(#in_token super::#path))
                }
            } else if let Some(ident) = path.get_ident() {
                match ident.to_string().as_ref() {
                    "crate" => parse_quote!(pub(crate)),
                    "self" => parse_quote! (pub(super)),
                    "super" => parse_quote! (pub(in super::super)),
                    _ => {
                        return Err(Error::new_spanned(ident, "unknown identifier"));
                    }
                }
            } else {
                return Err(Error::new_spanned(path, "path without `in` token"));
            }
        }
        vis => vis.clone(),
    };
    Ok(vis)
}

fn ident_append(ident: &Ident, suffix: &str) -> Ident {
    Ident::new(&format!("{}{}", ident, suffix), ident.span())
}

fn crate_name() -> Ident {
    let crate_name = proc_macro_crate::crate_name("newtype-enum");
    let crate_name = match &crate_name {
        Ok(n) => n,
        Err(_) => "newtype_enum",
    };
    Ident::new(crate_name, Span::call_site())
}
