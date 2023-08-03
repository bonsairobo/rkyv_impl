use rkyv::Archive;
use rkyv_impl::archive_impl;

#[derive(Archive)]
pub struct Foo;

#[archive_impl(blah)]
impl Foo {}

fn main() {}
