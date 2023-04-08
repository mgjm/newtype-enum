#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]

//! Provide the `#[newtype_enum]` attribute macro to derive the `Enum` and `Variant` traits from the `newtype-enum` crate.

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    meta, parse::Parser, parse_macro_input, parse_quote, token::Struct, Error, Fields, Generics,
    ItemEnum, ItemStruct, LitStr, Meta, Path, Variant, VisRestricted, Visibility,
};

/// Derive the `Enum` and `Variant` traits from the `newtype-enum` crate.
#[proc_macro_attribute]
pub fn newtype_enum(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    newtype_enum_impl(attr.into(), parse_macro_input!(item)).into()
}

macro_rules! unwrap_or_compile_error {
    ($expr:expr) => {
        match $expr {
            Ok(vis) => vis,
            Err(err) => return err.to_compile_error(),
        }
    };
}

fn newtype_enum_impl(meta: TokenStream, item: ItemEnum) -> TokenStream {
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
    fn new(meta: TokenStream, item: ItemEnum) -> Result<Self, Error> {
        let mut crate_name = crate_name();

        let mut variants = ident_append(&item.ident, "_variants");
        let mut variants_vis = item.vis.clone();

        if !meta.is_empty() {
            meta::parser(|meta| {
                let ident = meta
                    .path
                    .get_ident()
                    .ok_or_else(|| Error::new_spanned(&meta.path, "expected identifier"))?;
                if ident == "variants" {
                    let lit: LitStr = meta.value()?.parse()?;
                    let s = lit.value();
                    let s = if let Some(index) = s.rfind(' ') {
                        variants_vis = syn::parse_str(&s[..index])
                            .map_err(|err| Error::new_spanned(&lit, err))?;
                        &s[index + 1..]
                    } else {
                        &s
                    };
                    variants = syn::parse_str(s).map_err(|err| Error::new_spanned(&lit, err))?;
                } else if ident == "unstable_self_test" {
                    crate_name = parse_quote!(self);
                } else {
                    return Err(Error::new_spanned(ident, "unknown argument"));
                }
                Ok(())
            })
            .parse2(meta)?;
        }

        Ok(Self {
            item,
            crate_name,
            variants,
            variants_vis,
        })
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
                let doc = format!("See [`{ident}`]({variants}/struct.{ident}.html).");
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
        let items = self.item.variants.iter().filter(
            |var| !matches!(&var.fields, Fields::Unnamed(fields) if fields.unnamed.len() == 1),
        );

        if items.clone().next().is_none() {
            return TokenStream::new();
        }

        let vis = unwrap_or_compile_error!(super_vis(&self.item.vis, || parse_quote!(pub(super))));
        let item_attrs = self.item.attrs.iter().filter(
            |attr| !matches!(&attr.meta, Meta::NameValue(meta) if meta.path.is_ident("doc")),
        );

        let items = items.map(move |var| {
            let mut item = ItemStruct {
                attrs: item_attrs
                    .clone()
                    .chain(var.attrs.iter())
                    .cloned()
                    .collect(),
                vis: vis.clone(),
                struct_token: Struct::default(),
                ident: var.ident.clone(),
                generics: Generics::default(),
                fields: var.fields.clone(),
                semi_token: None,
            };
            match &mut item.fields {
                Fields::Unit => {
                    item.semi_token = parse_quote!(;);
                }
                Fields::Named(fields) => {
                    for field in &mut fields.named {
                        field.vis = unwrap_or_compile_error!(super_vis(&field.vis, || vis.clone()));
                    }
                }
                Fields::Unnamed(_) => {
                    return Error::new_spanned(var, "unsupported variant type").to_compile_error();
                }
            };
            quote!(#item)
        });

        let variants = &self.variants;
        let variants_vis = &self.variants_vis;
        let doc = format!("The generated variants of the `{}` enum.", self.item.ident);
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

                    fn is_enum_variant(e: &#e) -> bool {
                        matches!(e, #v(_))
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
                    parse_quote!(pub(#in_token super::#path))
                }
            } else if let Some(ident) = path.get_ident() {
                match ident.to_string().as_ref() {
                    "crate" => parse_quote!(pub(crate)),
                    "self" => parse_quote!(pub(super)),
                    "super" => parse_quote!(pub(in super::super)),
                    _ => {
                        return Err(Error::new_spanned(ident, "unknown identifier"));
                    }
                }
            } else {
                return Err(Error::new_spanned(path, "path without `in` token"));
            }
        }
        Visibility::Public(v) => Visibility::Public(*v),
    };
    Ok(vis)
}

fn ident_append(ident: &Ident, suffix: &str) -> Ident {
    Ident::new(&format!("{ident}{suffix}"), ident.span())
}

fn crate_name() -> Path {
    match proc_macro_crate::crate_name("newtype-enum") {
        Ok(proc_macro_crate::FoundCrate::Name(crate_name)) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            parse_quote!(::#crate_name)
        }
        _ => parse_quote!(::newtype_enum),
    }
}
