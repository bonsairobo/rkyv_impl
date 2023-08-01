use rkyv::{
    ser::{
        serializers::{
            AlignedSerializer, AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch,
        },
        Serializer,
    },
    AlignedVec, Archive, Infallible, Serialize,
};
use rkyv_impl::*;
use std::iter::Sum;

#[derive(Archive, Serialize)]
struct Foo<T> {
    elements: Vec<T>,
}

#[archive_impl(transform_bounds(T))]
impl<T> Foo<T> {
    #[archive_method(transform_bounds(T))]
    fn sum<S>(&self) -> S
    where
        T: Clone,
        S: Sum<T>,
    {
        self.elements.iter().cloned().sum()
    }
}

fn main() {
    let foo = Foo {
        elements: vec![1, 2, 3],
    };

    // Serialize.
    let buf = AlignedVec::new();
    let scratch = FallbackScratch::new(HeapScratch::<0>::new(), AllocScratch::new());
    let mut serializer = CompositeSerializer::new(AlignedSerializer::new(buf), scratch, Infallible);
    serializer.serialize_value(&foo).unwrap();
    let (serializer, _, _) = serializer.into_components();
    let buf = serializer.into_inner();

    let archived_foo = unsafe { rkyv::archived_root::<Foo<u32>>(&buf) };

    assert_eq!(foo.sum::<u32>(), 6);
    assert_eq!(archived_foo.sum::<u32>(), 6);
}
