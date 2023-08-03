#![deny(missing_docs)]
//! DOCS

use rkyv::Archive;
use rkyv_impl::archive_impl;

#[allow(missing_docs)]
#[derive(Archive)]
pub struct Foo;

#[archive_impl]
#[allow(missing_docs)]
impl Foo {
    pub fn bar() {}
}
