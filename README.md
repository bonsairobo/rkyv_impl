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
    #[archive_method(transform_bounds(T))]
    fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>
    {
        self.elements.iter().cloned().sum()
    }
}

// Notice that the trait bounds are transformed so that
// `T` is replaced with `T::Archived`.
fn call_generated_method<T, S>(foo: &ArchivedFoo<T>)
where
    T: Archive,
    T::Archived: Clone,
    S: Sum<T::Archived>
{
    let _ = foo.sum::<S>();
}
```

License: MIT/Apache-2.0
