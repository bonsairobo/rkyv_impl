use rkyv::Archive;
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive)]
pub struct Foo<R, T> {
    elements1: Vec<R>,
    elements2: Vec<T>,
}

#[archive_impl(transform_bounds(R, T))]
impl<R, T> Foo<R, T> {
    #[archive_method(transform_bounds(R, T))]
    pub fn sum<S>(&self) -> S
    where
        R: Clone,
        T: Clone,
        S: Sum<R>,
        S: Sum<T>,
        S: std::ops::Add<Output = S>,
    {
        self.elements1.iter().cloned().sum::<S>() + self.elements2.iter().cloned().sum::<S>()
    }
}

pub fn call_archived<R, T, S>(foo: ArchivedFoo<R, T>) -> S
where
    R: Archive,
    T: Archive,
    R::Archived: Clone,
    T::Archived: Clone,
    S: Sum<R::Archived>,
    S: Sum<T::Archived>,
    S: std::ops::Add<Output = S>,
{
    foo.sum::<S>()
}
