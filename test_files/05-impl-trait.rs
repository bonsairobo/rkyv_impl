use rkyv::Archive;
use rkyv_impl::archive_impl;

#[derive(Archive)]
pub struct Foo<T> {
    field: Vec<T>,
}

trait GetSlice<T> {
    fn get_slice(&self) -> &[T];
}

#[archive_impl(add_bounds(T: Archive<Archived = T>))]
impl<T> GetSlice<T> for Foo<T> {
    fn get_slice(&self) -> &[T] {
        &self.field
    }
}

fn call_archived<T: Archive<Archived = T>>(foo: ArchivedFoo<T>) {
    let _: &[T] = foo.get_slice();
}

fn main() {}
