use core::{ffi::CStr, fmt::Debug};

use elvwf::prelude::*;

fn roundtrip<'a, T>(msg: T, buf: &'a mut [u8]) -> (T, usize)
where
    T: Wired<'a> + Clone + PartialEq + Debug,
{
    let len = {
        let writer = &mut &mut buf[..];
        let total = writer.len();
        elvwf::msg::noprefix::encode(writer, msg.clone()).unwrap();
        total - writer.len()
    };

    let reader = &mut &buf[..];
    let value = elvwf::msg::noprefix::decode(reader, len).unwrap();
    (value, len)
}

fn assert_roundtrip<'a, T>(msg: T, buf: &'a mut [u8]) -> usize
where
    T: Wired<'a> + Clone + PartialEq + Debug,
{
    let original = msg.clone();
    let (decoded, len) = roundtrip(msg, buf);
    assert_eq!(original, decoded);
    len
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|C|_:6")))]
struct ScalarFormat {
    #[wf(scalar(format(le)))]
    a: u32,

    #[wf(scalar(format(be)), opt(trigger = B))]
    b: Option<u32>,

    #[wf(scalar(format(vle)), opt(if = 4, trigger = C))]
    c: u32,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "Z|Y|_:14|A:16|B:16|C:16")))]
struct ScalarSlot {
    #[wf(scalar(slot = A))]
    a: u16,

    #[wf(scalar(slot = B), opt(trigger = Z))]
    b: Option<u16>,

    #[wf(scalar(slot = C), opt(if = 4, trigger = Y))]
    c: u16,
}

#[test]
fn pass_scalar() {
    let mut data = [0u8; 32];

    for _ in 0..1000 {
        let w = &mut data[..];

        assert_roundtrip(
            ScalarFormat::randomized(&mut &mut [][..]).expect("src not large enough"),
            w,
        );

        assert_roundtrip(
            ScalarSlot::randomized(&mut &mut [][..]).expect("src not large enough"),
            w,
        );
    }
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|C|_:6")))]
struct ByteSlicePrefixed<'a> {
    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)))]
    a: &'a [u8],

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a [u8]>,

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(if = &[1, 2, 3], trigger = C))]
    c: &'a [u8],
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "Z|Y|_:14|A:16|B:16|C:16")))]
struct ByteSliceSlot<'a> {
    #[wf(slice(len(slot = A), bounded(low = 0, high = 16)))]
    a: &'a [u8],

    #[wf(slice(len(slot = B), bounded(low = 0, high = 16)), opt(trigger = Z))]
    b: Option<&'a [u8]>,

    #[wf(slice(len(slot = C), bounded(low = 0, high = 16)), opt(if = &[1, 2, 3], trigger = Y))]
    c: &'a [u8],
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct ByteSliceRemaining1<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)))]
    a: &'a [u8],
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|_:7")))]
struct ByteSliceRemaining2<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a [u8]>,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "C|_:7")))]
struct ByteSliceRemaining3<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(if = &[1,2 ,3], trigger = C))]
    c: &'a [u8],
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|C|_:6")))]
struct ByteSliceEmbedded<'a> {
    #[wf(slice(len(embedded)))]
    a: &'a [u8; 8],

    #[wf(slice(len(embedded)), opt(trigger = B))]
    b: Option<&'a [u8; 3]>,

    #[wf(slice(len(embedded)), opt(if  = &[1, 2, 3, 4, 5], trigger = C))]
    c: &'a [u8; 5],
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|C|_:6")))]
struct StrSlicePrefixed<'a> {
    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)))]
    a: &'a str,

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a str>,

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(if = { "abcdef" }, trigger = C))]
    c: &'a str,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "Z|Y|_:14|A:16|B:16|C:16")))]
struct StrSliceSlot<'a> {
    #[wf(slice(len(slot = A), bounded(low = 0, high = 16)))]
    a: &'a str,

    #[wf(slice(len(slot = B), bounded(low = 0, high = 16)), opt(trigger = Z))]
    b: Option<&'a str>,

    #[wf(slice(len(slot = C), bounded(low = 0, high = 16)), opt(if = { "abcdef" }, trigger = Y))]
    c: &'a str,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct StrSliceRemaining1<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)))]
    a: &'a str,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|_:7")))]
struct StrSliceRemaining2<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a str>,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "C|_:7")))]
struct StrSliceRemaining3<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(if = { "abcdef" }, trigger = C))]
    c: &'a str,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|C|_:6")))]
struct CStrSlicePrefixed<'a> {
    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)))]
    a: &'a CStr,

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a CStr>,

    #[wf(slice(len(prefixed(le)), bounded(low = 0, high = 16)), opt(if = { c"abcdef" }, trigger = C))]
    c: &'a CStr,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "Z|Y|_:14|A:16|B:16|C:16")))]
struct CStrSliceSlot<'a> {
    #[wf(slice(len(slot = A), bounded(low = 0, high = 16)))]
    a: &'a CStr,

    #[wf(slice(len(slot = B), bounded(low = 0, high = 16)), opt(trigger = Z))]
    b: Option<&'a CStr>,

    #[wf(slice(len(slot = C), bounded(low = 0, high = 16)), opt(if = { c"abcdef" }, trigger = Y))]
    c: &'a CStr,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct CStrSliceRemaining1<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)))]
    a: &'a CStr,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "B|_:7")))]
struct CStrSliceRemaining2<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(trigger = B))]
    b: Option<&'a CStr>,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "C|_:7")))]
struct CStrSliceRemaining3<'a> {
    #[wf(slice(len(remaining), bounded(low = 0, high = 16)), opt(if = { c"abcdef" }, trigger = C))]
    c: &'a CStr,
}

#[test]
fn pass_slice() {
    let mut src = [0u8; 512];
    let mut data = [0u8; 512];

    for _ in 0..1000 {
        // Byte slice
        let w = &mut data[..];

        let msg = ByteSlicePrefixed::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = ByteSliceSlot::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = ByteSliceRemaining1::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = ByteSliceRemaining2::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = ByteSliceRemaining3::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = ByteSliceEmbedded::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        // Str slice
        let w = &mut data[..];

        let msg = StrSlicePrefixed::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = StrSliceSlot::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = StrSliceRemaining1::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = StrSliceRemaining2::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = StrSliceRemaining3::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        // CStr slice
        let w = &mut data[..];

        let msg = CStrSlicePrefixed::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = CStrSliceSlot::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = CStrSliceRemaining1::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = CStrSliceRemaining2::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);

        let msg = CStrSliceRemaining3::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, w);
    }
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(align(u16)))]
struct AlignmentU16Array<'a>(
    #[wf(slice(len(prefixed(vle)), bounded(low = 0, high = 64)))] &'a [u8],
);

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(align(u32)))]
struct AlignmentU32Array<'a>(
    #[wf(slice(len(prefixed(vle)), bounded(low = 0, high = 64)))] &'a [u8],
);

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(align(u64)))]
struct AlignmentU64Array<'a>(
    #[wf(slice(len(prefixed(vle)), bounded(low = 0, high = 64)))] &'a [u8],
);

#[test]
fn pass_alignment() {
    let mut src = [0u8; 512];
    let mut data = [0u8; 512];

    for _ in 0..1000 {
        let w = &mut data[..];

        // 16-bit
        let msg = AlignmentU16Array::randomized(&mut &mut src[..]).expect("src not large enough");
        let len = assert_roundtrip(msg, w);
        assert!(len % ::core::mem::size_of::<u16>() == 0);

        // 32-bit
        let msg = AlignmentU32Array::randomized(&mut &mut src[..]).expect("src not large enough");
        let len = assert_roundtrip(msg, w);
        assert!(len % ::core::mem::size_of::<u32>() == 0);

        // 64-bit
        let msg = AlignmentU64Array::randomized(&mut &mut src[..]).expect("src not large enough");
        let len = assert_roundtrip(msg, w);
        assert!(len % ::core::mem::size_of::<u64>() == 0);
    }
}

#[derive(Wired, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "A|B|C|_:5|S:8|ID:16=123456789")))]
struct Header;

#[test]
fn pass_header() {
    const {
        assert!(Header::WF_FLAG_A == 1 << 31);
        assert!(Header::WF_FLAG_B == 1 << 30);
        assert!(Header::WF_FLAG_C == 1 << 29);
        assert!(Header::WF_SLOT_S == (0b1111_1111) << 16);
        assert!(Header::WF_MASK_ID == (0b11111111_11111111));
        assert!(Header::ID == 123456789);
    }
}

#[derive(Wired, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "ID:32=1289348", format(ne))))]
struct HeaderNe;

#[derive(Wired, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "ID:32=1289348", format(le))))]
struct HeaderLe;

#[derive(Wired, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "ID:32=1289348", format(be))))]
struct HeaderBe;

#[test]
fn pass_header_format() {
    let mut data = [0u8; 4];

    // native-endian
    let msg = HeaderNe;
    assert_roundtrip(msg, &mut data[..]);
    assert!(u32::from_le_bytes(data) == 1289348);

    // little-endian
    let msg = HeaderLe;
    assert_roundtrip(msg, &mut data[..]);
    assert!(u32::from_ne_bytes(data) == 1289348);

    // big-endian
    let msg = HeaderBe;
    assert_roundtrip(msg, &mut data[..]);
    assert!(u32::from_be_bytes(data) == 1289348);
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "S:8", offset = 7)))]
struct HeaderOffset<'a>(#[wf(slice(len(slot = S)))] &'a [u8; 12]);

#[test]
fn pass_header_offset() {
    let mut src = [0u8; 16];
    let mut data = [0u8; 64];

    for _ in 0..1000 {
        let msg = HeaderOffset::randomized(&mut &mut src[..]).expect("src not large enough");
        let h = elvwf::msg::header(&msg).unwrap();
        assert!(h as usize == msg.0.len() + 7);
        assert_roundtrip(msg, &mut data[..]);
    }
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "A|_:7")))]
struct HeaderFlagA(#[wf(scalar(format(le)), opt(trigger = A))] Option<u8>);

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "_:2|B|_:5")))]
struct HeaderFlagB(#[wf(scalar(format(le)), opt(trigger = B))] Option<u16>);

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
#[wf(struct(header(dsl = "_:2|S:6")))]
struct HeaderParent<'a> {
    #[wf(msg(len(embedded), header(flattened(shift = 0))))]
    a: HeaderFlagA,
    #[wf(msg(len(embedded), header(flattened(shift = 1))))]
    b: HeaderFlagB,

    #[wf(slice(len(slot = S), bounded(low = 0, high = 63)))]
    c: &'a [u8],
}

#[test]
fn pass_header_flattening() {
    let mut src = [0u8; 128];
    let mut data = [0u8; 512];

    for _ in 0..1000 {
        let msg = HeaderParent::randomized(&mut &mut src[..]).expect("src not large enough");
        let h = elvwf::msg::header(&msg).unwrap();
        assert!(elvwf::header::has(h, HeaderFlagA::WF_FLAG_A) == msg.a.0.is_some());
        assert!(elvwf::header::has(h, HeaderFlagB::WF_FLAG_B << 1) == msg.b.0.is_some());
        assert_roundtrip(msg, &mut data[..]);
    }
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct NewBuffer<'a>(#[wf(slice(len(remaining), bounded(low = 8, high = 16)))] &'a str);

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct Inner<'a> {
    #[wf(scalar(format(vle)))]
    id: u32,

    #[wf(msg(len(remaining)))]
    payload: NewBuffer<'a>,
}

#[derive(Wired, Randomized, Clone, Debug, PartialEq)]
struct Outer<'a> {
    #[wf(scalar(format(vle)))]
    class: u32,

    #[wf(msg(len(remaining)))]
    inner: Inner<'a>,
}

#[test]
fn pass_remaining() {
    let mut src = [0u8; 128];
    let mut data = [0u8; 512];

    for _ in 0..1000 {
        let msg = Outer::randomized(&mut &mut src[..]).expect("src not large enough");
        assert_roundtrip(msg, &mut data[..]);
    }
}
