#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::complexity, clippy::perf, clippy::style, clippy::pedantic)]

#![warn(missing_docs)]

use proc_macro::TokenStream as BaseTokenStream;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{GenericParam, ItemTrait, TypeParam};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn marker_trait(_: BaseTokenStream, input: BaseTokenStream) -> BaseTokenStream {
    let mut output = TokenStream::from(input.clone());
    syn::parse_macro_input!(input as MarkerTrait).to_tokens(&mut output);
    output.into()
}

struct MarkerTrait(ItemTrait);

impl MarkerTrait {
    fn to_tokens(self, tokens: &mut TokenStream) {
        let ItemTrait {
            unsafety,
            ident,
            mut generics,
            supertraits,
            ..
        } = self.0;

        let out_ident = Ident::new("__MarkerTrait__", Span::call_site());
        generics.params.push({
            let bounds = supertraits.clone();

            GenericParam::Type(TypeParam {
                attrs: Vec::new(),
                ident: out_ident.clone(),
                colon_token: None,
                bounds,
                eq_token: None,
                default: None,
            })
        });


        tokens.append(Punct::new('#', Spacing::Joint));
        tokens.append(Group::new(Delimiter::Bracket, quote!(automatically_derived)));
        unsafety.to_tokens(tokens);
        tokens.append(Ident::new("impl", Span::call_site()));

        let (g1, _, g3) = generics.split_for_impl();
        g1.to_tokens(tokens);
        tokens.append(ident);
        tokens.append(Ident::new("for", Span::call_site()));
        tokens.append(out_ident);
        g3.to_tokens(tokens);

        tokens.append(Group::new(Delimiter::Brace, TokenStream::new()));
    }
}

impl Parse for MarkerTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_def = input.parse::<ItemTrait>()?;
        if let Some(token) = trait_def.auto_token {
            Err(syn::Error::new_spanned(token, "auto trait is not allowed"))
        } else if !trait_def.items.is_empty() {
            let span = trait_def.items
                .iter()
                .map(Spanned::span)
                .reduce(move |acc, next| {
                    acc.join(next).expect("Trait item contents' tokens somehow point to different files")
                })
                .unwrap();

            Err(syn::Error::new(span, "Trait may not have any content"))
        } else {
            Ok(Self(trait_def))
        }
    }
}
