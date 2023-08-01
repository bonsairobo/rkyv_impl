# rkyv_impl

Copy `impl Foo` blocks into `impl ArchivedFoo`.

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
