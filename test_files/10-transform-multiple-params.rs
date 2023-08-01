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
struct Foo<R, T> {
    elements1: Vec<R>,
    elements2: Vec<T>,
}

#[archive_impl(transform_bounds(R, T))]
impl<R, T> Foo<R, T> {
    #[archive_method(transform_bounds(R, T))]
    fn sum<S>(&self) -> S
    where
        R: Clone,
        T: Clone,
        S: Sum<R>,
        S: Sum<T>,
        S: std::ops::Add<Output = S>,
    {
        self.elements1.iter().cloned().sum::<S>() + self.elements2.iter().cloned().sum::<S>()
    }
}

fn main() {
    let foo = Foo {
        elements1: vec![1, 2, 3],
        elements2: vec![4, 5, 6],
    };

    // Serialize.
    let buf = AlignedVec::new();
    let scratch = FallbackScratch::new(HeapScratch::<0>::new(), AllocScratch::new());
    let mut serializer = CompositeSerializer::new(AlignedSerializer::new(buf), scratch, Infallible);
    serializer.serialize_value(&foo).unwrap();
    let (serializer, _, _) = serializer.into_components();
    let buf = serializer.into_inner();

    let archived_foo = unsafe { rkyv::archived_root::<Foo<u32, u32>>(&buf) };

    assert_eq!(foo.sum::<u32>(), 21);
    assert_eq!(archived_foo.sum::<u32>(), 21);
}
