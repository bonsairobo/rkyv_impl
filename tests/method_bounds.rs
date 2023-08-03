use rkyv::Archive;
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive)]
pub struct Foo<T> {
    elements: Vec<T>,
}

#[archive_impl(add_bounds(T: Archive<Archived = T>))]
impl<T> Foo<T> {
    pub fn get_slice(&self) -> &[T] {
        &self.elements
    }

    // Show that the generated impl also inherits the `T::Archived: Eq` bound.
    #[archive_method(transform_bounds(T))]
    pub fn element_eq(&self, index: usize, value: &T) -> bool
    where
        T: Eq,
    {
        self.elements[index].eq(value)
    }

    // Show that the generated impl also inherits the `T::Archived: Clone`
    // bound.
    #[archive_method(transform_bounds(T))]
    pub fn clone_element(&self, index: usize) -> T
    where
        T: Clone,
    {
        self.elements[index].clone()
    }

    #[archive_method(transform_bounds(T))]
    pub fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>,
    {
        self.elements.iter().cloned().sum()
    }
}

pub fn call_archived_get_slice<T: Archive<Archived = T>>(foo: ArchivedFoo<T>) {
    let _: &[T] = foo.get_slice();
}

pub fn call_archived_element_eq<T: Archive<Archived = T>>(foo: ArchivedFoo<T>, expected_value: T)
where
    T: Eq,
{
    let _: bool = foo.element_eq(0, &expected_value);
}

pub fn call_archived_clone_element<T: Archive<Archived = T>>(foo: ArchivedFoo<T>)
where
    T: Clone,
{
    let _: T = foo.clone_element(0);
}

pub fn call_archived_sum<T, S>(foo: ArchivedFoo<T>) -> S
where
    T: Archive<Archived = T>,
    T: Clone,
    S: Sum<T::Archived>,
{
    foo.sum()
}
