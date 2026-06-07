use elvwf::prelude::*;

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(union(discriminant = ID))]
enum Request<'a, const N: usize> {
    Put(Put<'a, 8>),
    Get(Get),
    Remove(Remove<'a, N>),
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(struct(header(dsl = "ID:8=0x1")))]
struct Put<'a, const N: usize> {
    #[wf(slice(len(embedded)))]
    payload: &'a [u8; N],
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(struct(header(dsl = "ID:8=0x2")))]
struct Get;

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(struct(header(dsl = "ID:8=0x3")))]
struct Remove<'a, const N: usize> {
    #[wf(slice(len(embedded)))]
    payload: &'a [u8; N],
}

fn main() {
    for _ in 0..1000 {
        let mut src = [0u8; 128];
        let msg = elvwf::msg::randomized::<Request<4>>(&mut &mut src[..]).unwrap();

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Request<4>, elvwf::VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Request<4>, elvwf::VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
