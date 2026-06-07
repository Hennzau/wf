use elvwf::prelude::*;

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "S:16|D:16")))]
struct Attachment<'a, const N: usize> {
    #[wf(slice(len(slot = S), bounded(low = 1, high = 18)))]
    content: &'a str,

    #[wf(scalar(format(le), bounded(low = 0, high = u32::MAX - 1)))]
    value: u32,

    #[wf(scalar(slot = D, bounded(low = 1, high = u16::MAX as u32)))]
    data: u32,

    #[wf(slice(len(embedded)))]
    extra: &'a [u8; N],
}

fn main() {
    for _ in 0..1000 {
        let mut src = [0u8; 128];
        let msg = elvwf::msg::randomized::<Attachment<1>>(&mut &mut src[..]).unwrap();

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Attachment<1>, elvwf::VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Attachment<1>, elvwf::VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
