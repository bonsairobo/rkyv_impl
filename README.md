# rkyv_impl

Copy `impl T` blocks into `impl ArchivedT`.

```rust
use rkyv::Archive;
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive)]
struct Foo<T> {
    elements: Vec<T>
}

#[archive_impl(bounds(T: Archive))]
impl<T> Foo<T> {
    #[archive_method(bounds(T::Archived: Clone, S: Sum<T::Archived>))]
    fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>
    {
        self.elements.iter().cloned().sum()
    }
}

fn use_generated_method(foo: &ArchivedFoo<u32>) {
    let _ = foo.sum::<u32>();
}
```
