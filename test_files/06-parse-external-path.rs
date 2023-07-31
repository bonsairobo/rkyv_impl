use rkyv::Archive;

#[derive(Archive)]
pub struct Foo;

mod foo {
    use rkyv_impl::archive_impl;

    #[archive_impl]
    impl super::Foo {}
}

fn main() {}
