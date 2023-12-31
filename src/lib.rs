//! Implement a blanket implementation for a marker trait.
//!
//! [![MASTER CI status](https://github.com/Alorel/marker_trait-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Alorel/marker_trait-rs/actions/workflows/ci.yml?query=branch%3Amaster)
//! [![crates.io badge](https://img.shields.io/crates/v/marker_trait)](https://crates.io/crates/marker_trait)
//! [![docs.rs badge](https://img.shields.io/docsrs/marker_trait?label=docs.rs)](https://docs.rs/marker_trait)
//! [![dependencies badge](https://img.shields.io/librariesio/release/cargo/marker_trait)](https://libraries.io/cargo/marker_trait)
//!
//! # Basic Example
//!
//! ```
//! #[marker_trait::marker_trait]
//! pub trait AsyncTask: Send + 'static {}
//!
//! struct MySendStatic;
//! static_assertions::assert_impl_all!(MySendStatic: Send, AsyncTask);
//! ```
//!
//! Generated output:
#![cfg_attr(doctest, doc = " ````no_test")]
//! ```
//! pub trait AsyncTask: Send + 'static {}
//! impl<T: Send + 'static> AsyncTask for T {}
//! ````
//!
//! # Sealed example
//!
//! ```
//! #[marker_trait::marker_trait(sealed)]
//! pub trait AsyncTask: Send + 'static {}
//!
//! struct MySendStatic;
//! static_assertions::assert_impl_all!(MySendStatic: Send, AsyncTask, __SealModuleForAsyncTask__::Sealed);
//! ```
//!
//! Generated output:
#![cfg_attr(doctest, doc = " ````no_test")]
//! ```
//! pub trait AsyncTask: Send + 'static + __SealModuleForAsyncTask__::Sealed {}
//!# #[allow(non_snake_case)]
//! mod __SealModuleForAsyncTask__ {
//!    use super::*;
//!
//!     impl<__AsyncTaskImplementor__> Sealed for __AsyncTaskImplementor__
//!       where __AsyncTaskImplementor__: Send + 'static {}
//!
//!     pub trait Sealed {}
//! }
//! #[automatically_derived]
//! impl<__MarkerTrait__: Send + 'static + __SealModuleForAsyncTask__::Sealed> AsyncTask for __MarkerTrait__ {}
//! ````

#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::complexity, clippy::perf, clippy::style, clippy::pedantic)]
#![warn(missing_docs)]

use proc_macro::TokenStream as BaseTokenStream;

use macroific::elements::SimpleAttr;
use macroific::prelude::*;
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, GenericParam, ItemTrait, PathSegment, Token, TraitBound, TraitBoundModifier,
    TypeParam, TypeParamBound,
};

/// # Options
///
/// - `#[marker_trait(sealed)]` - derive [`sealed`](https://docs.rs/sealed) on the implementation (applies it to the trait definition too)
///
/// See [crate-level docs](crate) for an example.
#[proc_macro_attribute]
pub fn marker_trait(opts_in: BaseTokenStream, input: BaseTokenStream) -> BaseTokenStream {
    let opts = if opts_in.is_empty() {
        Options::default()
    } else {
        parse_macro_input!(opts_in as Options)
    };

    parse_macro_input!(input as MarkerTrait)
        .into_tokens(opts)
        .into()
}

#[derive(Default, AttributeOptions)]
struct Options {
    sealed: bool,
}

struct MarkerTrait(ItemTrait);

impl MarkerTrait {
    pub fn into_tokens(mut self, options: Options) -> TokenStream {
        let appendage = self.produce_appended_output(options);
        let mut tokens = self.0.into_token_stream();
        tokens.extend(appendage);

        tokens
    }

    fn produce_appended_output(&mut self, Options { sealed }: Options) -> TokenStream {
        let ItemTrait {
            ref unsafety,
            ref ident,
            ref generics,
            ref mut supertraits,
            ..
        } = self.0;

        let mut generics = generics.clone();
        let mut tokens = TokenStream::new();

        if sealed {
            let mod_name = format_ident!("__SealModuleFor{}__", ident);

            SimpleAttr::AUTO_DERIVED.to_tokens(&mut tokens);
            tokens.extend(quote! { #[allow(non_snake_case)] mod #mod_name });

            tokens.append(Group::new(Delimiter::Brace, {
                let mut tokens = quote! { use super::*; impl };

                let param_name = format_ident!("__{}Implementor__", ident);
                let sealed_trait_name = Ident::create("Sealed");

                let mut cloned_generics = generics.clone();
                cloned_generics
                    .params
                    .push(make_generic_param(&param_name, supertraits));

                let (g1, _, g3) = cloned_generics.split_for_impl();

                g1.to_tokens(&mut tokens);
                sealed_trait_name.to_tokens(&mut tokens);
                tokens.append(Ident::create("for"));
                tokens.append(param_name);
                g3.to_tokens(&mut tokens);
                tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));

                tokens.extend({
                    let c = format!(
                        "Seals the [`{ident}`] trait to be implementable only within its module"
                    );
                    quote! { #[doc = #c] }
                });

                tokens.append(Ident::create("pub"));
                tokens.append(Ident::create("trait"));
                tokens.append(sealed_trait_name.clone());

                tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));

                tokens
            }));

            supertraits.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: [PathSegment::from(mod_name), Ident::create("Sealed").into()]
                        .into_iter()
                        .collect(),
                },
            }));
        }

        let g2 = generics.clone();
        let g2 = g2.split_for_impl().1;

        let out_ident = Ident::new("__MarkerTrait__", Span::call_site());

        generics
            .params
            .push(make_generic_param(&out_ident, supertraits));

        let (g1, _, g3) = generics.split_for_impl();

        SimpleAttr::AUTO_DERIVED.to_tokens(&mut tokens);
        unsafety.to_tokens(&mut tokens);

        tokens.append(Ident::create("impl"));
        g1.to_tokens(&mut tokens);
        ident.to_tokens(&mut tokens);
        g2.to_tokens(&mut tokens);
        tokens.append(Ident::create("for"));
        tokens.append(out_ident);
        g3.to_tokens(&mut tokens);
        tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));

        tokens
    }
}

impl Parse for MarkerTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_def = input.parse::<ItemTrait>()?;
        if let Some(token) = trait_def.auto_token {
            return Err(syn::Error::new_spanned(token, "auto trait is not allowed"));
        }

        if trait_def.supertraits.is_empty() {
            return Err(syn::Error::new_spanned(
                trait_def,
                "Expected at least one supertrait",
            ));
        }

        // Check for empty body
        let mut items = trait_def.items.iter();
        if let Some(first) = items.next() {
            let mut span = first.span();
            for next in items {
                if let Some(joined) = span.join(next.span()) {
                    span = joined;
                } else {
                    return Err(syn::Error::new_spanned(
                        next,
                        "Trait item contents' tokens somehow point to different files",
                    ));
                }
            }
        }

        Ok(Self(trait_def))
    }
}

impl Parse for Options {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        AttributeOptions::from_stream(input)
    }
}

fn make_generic_param(
    ident: &Ident,
    bounds: &Punctuated<TypeParamBound, Token![+]>,
) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: ident.clone(),
        colon_token: None,
        bounds: bounds.clone(),
        eq_token: None,
        default: None,
    })
}
