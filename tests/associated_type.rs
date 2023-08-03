use rkyv::Archive;
use rkyv_impl::*;

#[derive(Archive)]
pub struct Foo<T> {
    field: T,
}

pub trait MakeBar {
    type Bar;

    fn make_bar(&self) -> Self::Bar;
}

#[archive_impl(transform_bounds(T))]
impl<T: MakeBar> Foo<T>
where
    <T as MakeBar>::Bar: Into<u32>,
{
    pub fn get_bar_u32(&self) -> u32 {
        self.field.make_bar().into()
    }
}

pub fn call_archived<T: Archive>(foo: ArchivedFoo<T>) -> u32
where
    T::Archived: MakeBar,
    <T::Archived as MakeBar>::Bar: Into<u32>,
{
    foo.get_bar_u32()
}
