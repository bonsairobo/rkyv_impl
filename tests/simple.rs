use rkyv::{rend::u32_le, Archive};
use rkyv_impl::archive_impl;

#[derive(Archive)]
pub struct Foo {
    field: Vec<u32_le>,
}

#[archive_impl]
impl Foo {
    pub fn get_slice(&self) -> &[u32_le] {
        &self.field
    }

    pub fn get_first(&self) -> Option<&u32_le> {
        self.field.first()
    }
}

pub fn call_archived(foo: ArchivedFoo) {
    let _: &[u32_le] = foo.get_slice();
    let _: Option<&u32_le> = foo.get_first();
}
