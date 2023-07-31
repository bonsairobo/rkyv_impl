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

#[derive(Archive, Serialize)]
pub struct Foo<T> {
    field: Vec<T>,
}

#[archive_impl(bounds(T: Archive<Archived = T>))]
impl<T: Clone> Foo<T>
where
    T: Eq,
{
    fn get_slice(&self) -> &[T] {
        &self.field
    }

    // Show that the generated impl also inherits the `T: Eq` bound.
    fn element_eq(&self, index: usize, value: &T) -> bool {
        self.field[index].eq(value)
    }

    // Show that the generated impl also inherits the `T: Clone` bound.
    fn clone_element(&self, index: usize) -> T {
        self.field[index].clone()
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

    assert_eq!(archived_foo.get_slice(), foo.get_slice());
}
