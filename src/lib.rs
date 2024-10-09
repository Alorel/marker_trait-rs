//! Implement a blanket implementation for a marker trait.
//!
//! [![MASTER CI status](https://github.com/Alorel/marker_trait-rs/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/marker_trait-rs/actions/workflows/test.yml?query=branch%3Amaster)
//! [![crates.io badge](https://img.shields.io/crates/v/marker_trait)](https://crates.io/crates/marker_trait)
//! [![Coverage Status](https://coveralls.io/repos/github/Alorel/marker_trait-rs/badge.svg)](https://coveralls.io/github/Alorel/marker_trait-rs)
//! [![dependencies badge](https://img.shields.io/librariesio/release/cargo/marker_trait)](https://libraries.io/cargo/marker_trait)
//!

//! # Examples
//!
//! <details><summary>Basic example</summary>
//!
//! ```
//! #[marker_trait::marker_trait]
//! trait Cloneable: Clone + PartialEq {}
//!
//! #[derive(Clone, Eq, PartialEq, Debug)]
//! struct Wrapper<T>(T);
//!
//! fn acceptor<T: Cloneable>(value: T) -> T { value }
//!
//! assert_eq!(acceptor(Wrapper(1)), Wrapper(1)); // Compiles fine
//! ```
//!
//! Generated output:
#![cfg_attr(doctest, doc = " ````no_test")]
//! ```
//! trait Cloneable: Clone + PartialEq {}
//! impl<T: Clone + PartialEq> Cloneable for T {}
//! ````
//!
//! </details>

//! <details><summary>Generic example</summary>
//!
//! ```
//! trait MySuper<A, B>: AsRef<A> {
//!     type C;
//!
//!     fn foo(self) -> Result<B, Self::C>;
//! }
//!
//! #[marker_trait::marker_trait]
//! trait MySub<B, C>: MySuper<Self, B, C = C> + Sized {
//! }
//!
//! struct MyStruct;
//! impl AsRef<MyStruct> for MyStruct {
//!   fn as_ref(&self) -> &Self { self }
//! }
//! impl MySuper<MyStruct, i8> for MyStruct {
//!   type C = u8;
//!   fn foo(self) -> Result<i8, Self::C> { Err(u8::MAX) }
//! }
//!
//! fn acceptor<T: MySub<i8, u8>>(input: T) -> u8 { input.foo().unwrap_err() }
//!
//! assert_eq!(acceptor(MyStruct), u8::MAX);
//! ```
//!
//! Generated output:
//!
#![cfg_attr(doctest, doc = " ````no_test")]
//! ```
//! impl<B, C, __MarkerTrait__: MySuper<Self, B, C = C> + Sized> MySub<B, C> for __MarkerTrait__ {}
//! ````
//!
//! </details>

//! <details><summary>Failing examples</summary>
//!
//! ```compile_fail
//! #[marker_trait::marker_trait]
//! trait Cloneable: Clone {}
//!
//! struct NonClone;
//!
//! fn acceptor<T: Cloneable>(value: T) -> T { value }
//!
//! let _ = acceptor(NonClone); // Doesn't implement clone and therefore cloneable
//! ```
//!
//! ```compile_fail
//! #[marker_trait::marker_trait]
//! # #[allow(dead_code)]
//! trait MyTrait: AsRef<Self::Foo> { // Empty trait body expected
//!   type Foo;
//! }
//! ```
//!
//! ```compile_fail
//! #[marker_trait::marker_trait]
//! # #[allow(dead_code)]
//! trait Foo {} // Expected at least one supertrait
//! ```
//!
//! </details>

#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::complexity, clippy::perf, clippy::style, clippy::pedantic)]
#![warn(missing_docs)]

use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Error, GenericParam, ItemTrait, Token, TypeParamBound};

/// See [crate-level docs](crate) for an example.
#[proc_macro_attribute]
pub fn marker_trait(_: TokenStream1, input: TokenStream1) -> TokenStream1 {
    parse_macro_input!(input as MarkerTrait)
        .into_tokens()
        .into()
}

struct MarkerTrait(ItemTrait);

impl MarkerTrait {
    pub fn into_tokens(mut self) -> TokenStream {
        let appendage = self.produce_appended_output();
        let mut tokens = self.0.into_token_stream();
        tokens.extend(appendage);

        tokens
    }

    fn produce_appended_output(&mut self) -> TokenStream {
        let ItemTrait {
            ref unsafety,
            ref ident,
            ref generics,
            ref mut supertraits,
            ..
        } = self.0;

        let g2 = generics.split_for_impl().1;
        let mut generics = generics.clone();

        let out_ident = Ident::new("__MarkerTrait__", Span::call_site());

        generics
            .params
            .push(make_generic_param(&out_ident, supertraits));

        let (g1, _, g3) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            #[allow(clippy::all)]
            #unsafety impl #g1 #ident #g2 for #out_ident #g3 {}
        }
    }
}

impl Parse for MarkerTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const MSG_AUTO: &str = "auto trait is not allowed";
        const MSG_SUPER: &str = "Expected at least one supertrait";
        const MSG_EMPTY: &str = "Expected empty trait";

        let trait_def = input.parse::<ItemTrait>()?;

        if trait_def.auto_token.is_some() {
            return Err(Error::new(Span::call_site(), MSG_AUTO));
        }

        if trait_def.supertraits.is_empty() {
            return Err(Error::new(Span::call_site(), MSG_SUPER));
        }

        if !trait_def.items.is_empty() {
            return Err(Error::new(Span::call_site(), MSG_EMPTY));
        }

        Ok(Self(trait_def))
    }
}

fn make_generic_param(
    ident: &Ident,
    bounds: &Punctuated<TypeParamBound, Token![+]>,
) -> GenericParam {
    parse_quote!(#ident: #bounds)
}
