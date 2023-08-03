use rkyv::Archive;
use rkyv_impl::archive_impl;

#[derive(Archive)]
pub struct Foo {
    field: Vec<u32>,
}

#[archive_impl]
impl Foo {
    pub fn get_slice(&self) -> &[u32] {
        &self.field
    }

    pub fn get_first(&self) -> Option<&u32> {
        self.field.first()
    }
}

pub fn call_archived(foo: ArchivedFoo) {
    let _: &[u32] = foo.get_slice();
    let _: Option<&u32> = foo.get_first();
}
