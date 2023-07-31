use rkyv::{
    ser::{
        serializers::{
            AlignedSerializer, AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch,
        },
        Serializer,
    },
    AlignedVec, Archive, Infallible, Serialize,
};
use rkyv_impl::archive_impl;

#[derive(Archive, Serialize)]
pub struct Foo<T> {
    field: Vec<T>,
}

trait GetSlice<T> {
    fn get_slice(&self) -> &[T];
}

#[archive_impl]
impl<T> GetSlice<T> for Foo<T>
where
    T: Archive<Archived = T>,
{
    fn get_slice(&self) -> &[T] {
        &self.field
    }
}

fn main() {
    let foo = Foo {
        field: vec![1, 2, 3],
    };

    // Serialize.
    let buf = AlignedVec::new();
    let scratch = FallbackScratch::new(HeapScratch::<0>::new(), AllocScratch::new());
    let mut serializer = CompositeSerializer::new(AlignedSerializer::new(buf), scratch, Infallible);
    serializer.serialize_value(&foo).unwrap();
    let (serializer, _, _) = serializer.into_components();
    let buf = serializer.into_inner();

    let archived_foo = unsafe { rkyv::archived_root::<Foo<u32>>(&buf) };

    assert_eq!(GetSlice::get_slice(archived_foo), GetSlice::get_slice(&foo));
}
