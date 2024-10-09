<!-- cargo-rdme start -->

Implement a blanket implementation for a marker trait.

[![MASTER CI status](https://github.com/Alorel/marker_trait-rs/actions/workflows/test.yml/badge.svg)](https://github.com/Alorel/marker_trait-rs/actions/workflows/test.yml?query=branch%3Amaster)
[![crates.io badge](https://img.shields.io/crates/v/marker_trait)](https://crates.io/crates/marker_trait)
[![Coverage Status](https://coveralls.io/repos/github/Alorel/marker_trait-rs/badge.svg)](https://coveralls.io/github/Alorel/marker_trait-rs)
[![dependencies badge](https://img.shields.io/librariesio/release/cargo/marker_trait)](https://libraries.io/cargo/marker_trait)

# Examples

<details><summary>Basic example</summary>

```rust
#[marker_trait::marker_trait]
trait Cloneable: Clone + PartialEq {}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Wrapper<T>(T);

fn acceptor<T: Cloneable>(value: T) -> T { value }

assert_eq!(acceptor(Wrapper(1)), Wrapper(1)); // Compiles fine
```

Generated output:
```rust
trait Cloneable: Clone + PartialEq {}
impl<T: Clone + PartialEq> Cloneable for T {}
````

</details>
<details><summary>Generic example</summary>

```rust
trait MySuper<A, B>: AsRef<A> {
    type C;

    fn foo(self) -> Result<B, Self::C>;
}

#[marker_trait::marker_trait]
trait MySub<B, C>: MySuper<Self, B, C = C> + Sized {
}

struct MyStruct;
impl AsRef<MyStruct> for MyStruct {
  fn as_ref(&self) -> &Self { self }
}
impl MySuper<MyStruct, i8> for MyStruct {
  type C = u8;
  fn foo(self) -> Result<i8, Self::C> { Err(u8::MAX) }
}

fn acceptor<T: MySub<i8, u8>>(input: T) -> u8 { input.foo().unwrap_err() }

assert_eq!(acceptor(MyStruct), u8::MAX);
```

Generated output:

```rust
impl<B, C, __MarkerTrait__: MySuper<Self, B, C = C> + Sized> MySub<B, C> for __MarkerTrait__ {}
````

</details>
<details><summary>Failing examples</summary>

```rust
#[marker_trait::marker_trait]
trait Cloneable: Clone {}

struct NonClone;

fn acceptor<T: Cloneable>(value: T) -> T { value }

let _ = acceptor(NonClone); // Doesn't implement clone and therefore cloneable
```

```rust
#[marker_trait::marker_trait]
trait MyTrait: AsRef<Self::Foo> { // Empty trait body expected
  type Foo;
}
```

```rust
#[marker_trait::marker_trait]
trait Foo {} // Expected at least one supertrait
```

</details>

<!-- cargo-rdme end -->
