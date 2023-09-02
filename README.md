<!-- cargo-rdme start -->

Implement a blanket implementation for a marker trait.

[![MASTER CI status](https://github.com/Alorel/marker_trait-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Alorel/marker_trait-rs/actions/workflows/ci.yml?query=branch%3Amaster)
[![crates.io badge](https://img.shields.io/crates/v/marker_trait)](https://crates.io/crates/marker_trait)
[![docs.rs badge](https://img.shields.io/docsrs/marker_trait?label=docs.rs)](https://docs.rs/marker_trait)
[![dependencies badge](https://img.shields.io/librariesio/release/cargo/marker_trait)](https://libraries.io/cargo/marker_trait)

# Basic Example

```rust
#[marker_trait::marker_trait]
pub trait AsyncTask: Send + 'static {}

struct MySendStatic;
static_assertions::assert_impl_all!(MySendStatic: Send, AsyncTask);
```

Generated output:
```rust
pub trait AsyncTask: Send + 'static {}
impl<T: Send + 'static> AsyncTask for T {}
````

# Sealed example

```rust
#[marker_trait::marker_trait(sealed)]
pub trait AsyncTask: Send + 'static {}

struct MySendStatic;
static_assertions::assert_impl_all!(MySendStatic: Send, AsyncTask, __SealModuleForAsyncTask__::Sealed);
```

Generated output:
```rust
pub trait AsyncTask: Send + 'static + __SealModuleForAsyncTask__::Sealed {}
mod __SealModuleForAsyncTask__ {
   use super::*;

    impl<__AsyncTaskImplementor__> Sealed for __AsyncTaskImplementor__
      where __AsyncTaskImplementor__: Send + 'static {}

    pub trait Sealed {}
}
#[automatically_derived]
impl<__MarkerTrait__: Send + 'static + __SealModuleForAsyncTask__::Sealed> AsyncTask for __MarkerTrait__ {}
````

<!-- cargo-rdme end -->
