#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::complexity, clippy::perf, clippy::style, clippy::pedantic)]
#![warn(missing_docs)]

//! Macro for the [`marker_trait`](https://docs.rs/marker_trait) crate.

use proc_macro::TokenStream as BaseTokenStream;

use macroific::elements::SimpleAttr;
use macroific::prelude::*;
use proc_macro2::{Delimiter, Group, Ident, Punct, Span, TokenStream};
use quote::{format_ident, quote, TokenStreamExt, ToTokens};
use syn::{GenericParam, Generics, ItemTrait, parse_macro_input, PathSegment, PredicateType, Token, TraitBound, TraitBoundModifier, Type, TypeParam, TypeParamBound, TypePath, WherePredicate};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

/// # Options
///
/// - `#[marker_trait(sealed)]` - derive [`sealed`](https://docs.rs/sealed) on the implementation (applies it to the trait definition too)
#[proc_macro_attribute]
pub fn marker_trait(opts_in: BaseTokenStream, input: BaseTokenStream) -> BaseTokenStream {
    let opts = {
        if opts_in.is_empty() {
            Options::default()
        } else {
            parse_macro_input!(opts_in as Options)
        }
    };

    let mut output = TokenStream::from(input.clone());
    parse_macro_input!(input as MarkerTrait).into_tokens(opts, &mut output);

    output.into()
}

#[derive(Default, AttributeOptions)]
struct Options {
    sealed: bool,
}

struct MarkerTrait {
    unsafety: Option<Token![unsafe]>,
    ident: Ident,
    generics: Generics,
    supertraits: Punctuated<TypeParamBound, Token![+]>,
}

impl MarkerTrait {
    fn into_tokens(self, Options { sealed }: Options, tokens: &mut TokenStream) {
        let Self {
            unsafety,
            ident,
            mut generics,
            mut supertraits,
        } = self;

        if sealed {
            let mod_name = format_ident!("__SealModuleFor{}__", ident);

            SimpleAttr::AUTO_DERIVED.to_tokens(tokens);
            tokens.append(Ident::create("mod"));
            tokens.append(mod_name.clone());

            tokens.append(Group::new(Delimiter::Brace, {
                let mut tokens = quote! { use super::*; impl };

                let param_name = format_ident!("__{}Implementor__", ident);
                let sealed_trait_name = Ident::create("Sealed");

                let mut cloned_generics = generics.clone();
                cloned_generics.params.push(GenericParam::Type(TypeParam {
                    attrs: Vec::new(),
                    ident: param_name.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                    eq_token: None,
                    default: None,
                }));
                cloned_generics.make_where_clause().predicates.push(WherePredicate::Type(PredicateType {
                    lifetimes: None,
                    bounded_ty: Type::Path(TypePath {
                        qself: None,
                        path: param_name.clone().into(),
                    }),
                    colon_token: Default::default(),
                    bounds: supertraits.clone(),
                }));

                let (g1, _, g3) = cloned_generics.split_for_impl();

                let (_, g2, _) = generics.split_for_impl();
                g1.to_tokens(&mut tokens);
                sealed_trait_name.to_tokens(&mut tokens);
                tokens.append(Ident::create("for"));
                g2.to_tokens(&mut tokens);

                tokens.append(param_name);

                g3.to_tokens(&mut tokens);
                tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));

                tokens.extend({
                    let c = format!("Seals the [`{}`] trait to be implementable only within its module", ident);
                    quote! { #[doc = #c] }
                });


                tokens.append(Ident::create("pub"));
                tokens.append(Ident::create("trait"));
                tokens.append(sealed_trait_name.clone());
                tokens.append(Punct::new_alone(':'));
                supertraits.to_tokens(&mut tokens);

                tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));

                tokens
            }));

            supertraits.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: [
                        PathSegment::from(mod_name),
                        Ident::create("Sealed").into(),
                    ].into_iter().collect(),
                },
            }));
        }

        let g2 = generics.clone();
        let g2 = g2.split_for_impl().1;

        let out_ident = Ident::new("__MarkerTrait__", Span::call_site());

        generics.params.push(GenericParam::Type(TypeParam {
            attrs: Vec::new(),
            ident: out_ident.clone(),
            colon_token: None,
            bounds: supertraits,
            eq_token: None,
            default: None,
        }));

        let (g1, _, g3) = generics.split_for_impl();

        SimpleAttr::AUTO_DERIVED.to_tokens(tokens);
        unsafety.to_tokens(tokens);

        tokens.append(Ident::create("impl"));
        g1.to_tokens(tokens);
        tokens.append(ident);
        g2.to_tokens(tokens);
        tokens.append(Ident::create("for"));
        tokens.append(out_ident);
        g3.to_tokens(tokens);
        tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));
    }
}

impl Parse for MarkerTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_def = input.parse::<ItemTrait>()?;
        if let Some(token) = trait_def.auto_token {
            return Err(syn::Error::new_spanned(token, "auto trait is not allowed"));
        }

        if trait_def.supertraits.is_empty() {
            return Err(syn::Error::new_spanned(trait_def, "Expected at least one supertrait"));
        }

        // Check for empty body
        let mut items = trait_def.items.into_iter();
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

        Ok(Self {
            unsafety: trait_def.unsafety,
            ident: trait_def.ident,
            generics: trait_def.generics,
            supertraits: trait_def.supertraits,
        })
    }
}

impl Parse for Options {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        AttributeOptions::from_stream(input)
    }
}
