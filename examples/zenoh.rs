use elvwf::prelude::*;

#[derive(Wired, Randomized, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[wf(enum(repr(u8)))]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(struct(header(dsl = "_:6|_|N", format(be))))]
pub struct WireExpr<'a> {
    #[wf(scalar(format(vle)))]
    scope: u16,
    #[wf(msg(len(embedded), header(flattened(shift = 1))))]
    mapping: Mapping,
    #[wf(slice(len(prefixed(vle)), bounded(low = 0, high = 64)), opt(if = { "" }, trigger = N))]
    suffix: &'a str,
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq, Eq)]
#[wf(struct(header(dsl = "Z|_:2|ID:5=0x1d", format(be))))]
struct Push<'a> {
    #[wf(msg(len(embedded), header(flattened(shift = 5))))]
    wire_expr: WireExpr<'a>,
}

fn main() {
    for _ in 0..1000 {
        let mut src = [0u8; 512];
        let msg = elvwf::msg::randomized::<Push>(&mut &mut src[..]).unwrap();

        let mut data = [0u8; 512];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Push, VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Push, VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
