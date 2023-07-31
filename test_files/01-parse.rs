use rkyv::Archive;
use rkyv_impl::archive_impl;

#[derive(Archive)]
pub struct Foo;

#[archive_impl]
impl Foo {}

fn main() {}
