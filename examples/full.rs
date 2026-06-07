use elvwf::prelude::*;

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "S:16")))]
struct Attachment<'a, const N: usize> {
    #[wf(slice(len(slot = S), bounded(low = 0, high = 16)))]
    content: &'a str,

    #[wf(slice(len(embedded)))]
    extra: &'a [u8; N],
}

#[derive(Wired, Randomized, Clone, Copy, Debug, PartialEq)]
#[wf(enum(repr(u32), format(le)))]
enum MyEnum {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "_:6|A|B")))]
struct Inner {
    #[wf(scalar(format(le), bounded(low = 16, high = 256)), opt(trigger = A))]
    a: Option<u32>,
    #[wf(scalar(format(vle), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<u16>,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "A|B|C|D|E|F|_:2")))]
struct Message<'a, T> {
    #[wf(opt(trigger = A), scalar(format(vle)))]
    pub opt_scalar: Option<u32>,
    #[wf(opt(trigger = B), slice(len(prefixed(ne)), bounded(low = 0, high = 8)))]
    pub opt_slice: Option<&'a str>,
    #[wf(opt(trigger = C), msg(len(prefixed(ne))))]
    pub opt_msg: Option<Attachment<'a, 3>>,

    #[wf(scalar(format(be)))]
    pub scalar: u64,
    #[wf(slice(len(prefixed(ne))))]
    pub slice: &'a [u8; 10],
    #[wf(msg(len(prefixed(le))))]
    pub msg: Attachment<'a, 3>,

    #[wf(msg(len(embedded), header(flattened(shift = 0))))]
    pub inner: Inner,

    #[wf(opt(if = 0, trigger = D), scalar(format(ne)))]
    pub cond_scalar: u16,
    #[wf(opt(if = { "hello" }, trigger = E), slice(len(prefixed(le)), bounded(low = 3, high = 8)))]
    pub cond_slice: &'a str,
    #[wf(opt(if = Attachment { content: "hello", extra: &[] }, trigger = F))]
    #[wf(msg(len(prefixed(vle))))]
    pub cond_msg: Attachment<'a, 0>,

    #[wf(msg(len(prefixed(vle))))]
    pub extra: T,

    #[wf(msg(len(embedded)))]
    pub myenum: MyEnum,

    #[wf(slice(len(remaining), bounded(low = 0, high = 32)))]
    pub payload: &'a [u8],
}

fn main() {
    for _ in 0..1000 {
        let mut src = [0u8; 512];
        let msg = elvwf::msg::randomized::<Message<Attachment<3>>>(&mut &mut src[..]).unwrap();

        let mut data = [0u8; 512];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Message<Attachment<3>>, elvwf::VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Message<Attachment<3>>, elvwf::VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
