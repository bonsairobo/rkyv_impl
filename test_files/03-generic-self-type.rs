use rkyv::Archive;
use rkyv_impl::*;

#[derive(Archive)]
pub struct Foo<T> {
    field: Vec<T>,
}

#[archive_impl(add_bounds(T: Archive<Archived = T>))]
impl<T> Foo<T> {
    fn get_slice(&self) -> &[T] {
        &self.field
    }

    // Show that the generated impl also inherits the `T::Archived: Eq` bound.
    #[archive_method(transform_bounds(T))]
    fn element_eq(&self, index: usize, value: &T) -> bool
    where
        T: Eq,
    {
        self.field[index].eq(value)
    }

    // Show that the generated impl also inherits the `T::Archived: Clone`
    // bound.
    #[archive_method(transform_bounds(T))]
    fn clone_element(&self, index: usize) -> T
    where
        T: Clone,
    {
        self.field[index].clone()
    }
}

fn call_archived_get_slice<T: Archive<Archived = T>>(foo: ArchivedFoo<T>) {
    let _: &[T] = foo.get_slice();
}

fn call_archived_element_eq<T: Archive<Archived = T>>(foo: ArchivedFoo<T>, expected_value: T)
where
    T: Eq,
{
    let _: bool = foo.element_eq(0, &expected_value);
}

fn call_archived_clone_element<T: Archive<Archived = T>>(foo: ArchivedFoo<T>)
where
    T: Clone,
{
    let _: T = foo.clone_element(0);
}

fn main() {}
