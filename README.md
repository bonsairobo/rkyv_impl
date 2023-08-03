# rkyv_impl

[![Crates.io](https://img.shields.io/crates/v/rkyv_impl.svg)](https://crates.io/crates/rkyv_impl)
[![Docs.rs](https://docs.rs/rkyv_impl/badge.svg)](https://docs.rs/rkyv_impl)

Implement methods for `Foo` and `ArchivedFoo` in a single `impl` block.

```rust
use rkyv::Archive;
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive)]
struct Foo<T> {
    elements: Vec<T>
}

#[archive_impl(transform_bounds(T))]
impl<T> Foo<T> {
    // Notice that the where clause is transformed so that
    // `T` is replaced with `T::Archived` in the generated `impl`.
    #[archive_method(transform_bounds(T))]
    fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>
    {
        self.elements.iter().cloned().sum()
    }
}

fn use_generated_method(foo: &ArchivedFoo<u32>) {
    // Call the generated method!
    let _ = foo.sum::<u32>();
}
```

License: MIT/Apache-2.0
