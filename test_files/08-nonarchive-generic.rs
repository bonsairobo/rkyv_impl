use rkyv::Archive;
use rkyv_impl::*;

#[derive(Archive)]
pub struct Foo<T> {
    field: Vec<T>,
}

#[archive_impl(bounds(T: Archive<Archived = T>))]
impl<T> Foo<T> {
    fn get_slice(&self) -> &[T] {
        &self.field
    }
}

// This shows that the generated trait bounds do not apply to the original impl.
fn main() {
    struct NonArchive(u32);

    let foo = Foo {
        field: vec![NonArchive(1), NonArchive(2), NonArchive(3)],
    };

    let _ = foo.get_slice();
}
