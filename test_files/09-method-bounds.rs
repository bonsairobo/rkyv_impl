use rkyv::Archive;
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive)]
struct Foo<T> {
    elements: Vec<T>,
}

#[archive_impl(transform_bounds(T))]
impl<T> Foo<T> {
    #[archive_method(transform_bounds(T))]
    fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>,
    {
        self.elements.iter().cloned().sum()
    }
}

fn call_archived<T, S>(foo: ArchivedFoo<T>) -> S
where
    T: Archive,
    T::Archived: Clone,
    S: Sum<T::Archived>,
    S: std::ops::Add<Output = S>,
{
    foo.sum::<S>()
}

fn main() {}
