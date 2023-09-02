//! Implement a blanket implementation for a marker trait.
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
//! Uses the [`sealed`](https://docs.rs/sealed) crate
//!
//! ```
//! #[marker_trait::marker_trait(sealed)]
//! pub trait AsyncTask: Send + 'static {}
//!
//! struct MySendStatic;
//! // name generated by the sealed crate
//! static_assertions::assert_impl_all!(MySendStatic: Send, AsyncTask, __seal_async_task::Sealed);
//! ```
//!
//! Generated output:
#![cfg_attr(doctest, doc = " ````no_test")]
//! ```
//! #[::sealed::sealed]
//! pub trait AsyncTask: Send + 'static {}
//!
//! #[::sealed::sealed]
//! impl<T: Send + 'static> AsyncTask for T {}
//! ````

pub use marker_trait_macro::marker_trait;
