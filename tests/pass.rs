use elvwf::Wired;

fn roundtrip<'a, T>(msg: T, buf: &'a mut [u8]) -> (T, usize)
where
    T: Wired<'a> + Clone + PartialEq + std::fmt::Debug,
{
    let len = {
        let writer = &mut &mut buf[..];
        let total = writer.len();
        elvwf::msg::encode_value(writer, msg.clone()).unwrap();
        total - writer.len()
    };

    let reader = &mut &buf[..];
    let value = elvwf::msg::decode_value(reader, len).unwrap();
    (value, len)
}

fn assert_roundtrip<'a, T>(msg: T, buf: &'a mut [u8])
where
    T: Wired<'a> + Clone + PartialEq + std::fmt::Debug,
{
    let original = msg.clone();
    let (decoded, _) = roundtrip(msg, buf);
    assert_eq!(original, decoded);
}

#[derive(Wired)]
struct Empty;

#[test]
fn test_empty() {
    assert_eq!(elvwf::msg::value_len(&Empty), 0);
}

mod scalar_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct ScalarBe {
        #[wf(format = Be)]
        v: u32,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct ScalarLe {
        #[wf(format = Le)]
        v: u32,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct ScalarNe {
        #[wf(format = Ne)]
        v: u32,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct ScalarVle {
        #[wf(format = VLE)]
        v: u32,
    }

    #[test]
    fn test_be_u32() {
        let mut data = [0u8; 64];
        let msg = ScalarBe { v: 0x12345678 };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 4);
        assert_eq!(&data[..4], &[0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_le_u32() {
        let mut data = [0u8; 64];
        let msg = ScalarLe { v: 0x12345678 };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 4);
        assert_eq!(&data[..4], &[0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_ne_u32() {
        let mut data = [0u8; 64];
        let msg = ScalarNe { v: 0x12345678 };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_vle_small() {
        let mut data = [0u8; 64];
        let msg = ScalarVle { v: 0 };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_vle_max() {
        let mut data = [0u8; 64];
        let msg = ScalarVle { v: u32::MAX };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_vle_various() {
        let mut data = [0u8; 64];
        for v in [1u32, 127, 128, 16383, 16384, 2097151, 2097152] {
            let msg = ScalarVle { v };
            let (value, _) = roundtrip(msg.clone(), &mut data);
            assert_eq!(msg, value);
        }
    }
}

mod scalar_types {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct AllScalars {
        #[wf(format = Be)]
        a: u8,
        #[wf(format = Be)]
        b: u16,
        #[wf(format = Be)]
        c: u32,
        #[wf(format = Be)]
        d: u64,
        #[wf(format = Le)]
        e: i8,
        #[wf(format = Le)]
        f: i16,
        #[wf(format = Le)]
        g: i32,
        #[wf(format = Le)]
        h: i64,
    }

    #[test]
    fn test_all_scalar_types() {
        let mut data = [0u8; 128];
        let msg = AllScalars {
            a: 0xAB,
            b: 0xABCD,
            c: 0x12345678,
            d: 0x1122334455667788,
            e: -1,
            f: -2,
            g: -3,
            h: -4,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_signed_extremes() {
        let mut data = [0u8; 128];
        let msg = AllScalars {
            a: u8::MAX,
            b: u16::MAX,
            c: u32::MAX,
            d: u64::MAX,
            e: i8::MIN,
            f: i16::MIN,
            g: i32::MIN,
            h: i64::MIN,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod slice_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct PrefixedSlice<'a> {
        #[wf(len(prefixed(format = Be)))]
        data: &'a [u8],
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct PrefixedStr<'a> {
        #[wf(len(prefixed(format = Le)))]
        s: &'a str,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct FixedArray<'a> {
        #[wf(len(prefixed(format = Ne)))]
        arr: &'a [u8; 5],
    }

    #[test]
    fn test_prefixed_slice() {
        let mut data = [0u8; 64];
        let msg = PrefixedSlice {
            data: &[1, 2, 3, 4, 5],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_empty_slice() {
        let mut data = [0u8; 64];
        let msg = PrefixedSlice { data: &[] };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_prefixed_str() {
        let mut data = [0u8; 64];
        let msg = PrefixedStr { s: "Hello, World!" };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_empty_str() {
        let mut data = [0u8; 64];
        let msg = PrefixedStr { s: "" };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_unicode_str() {
        let mut data = [0u8; 128];
        let msg = PrefixedStr {
            s: "héllo 世界 🦀"
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_fixed_array() {
        let mut data = [0u8; 64];
        let msg = FixedArray {
            arr: &[10, 20, 30, 40, 50],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod nested_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct Inner<'a> {
        #[wf(format = Be)]
        id: u32,
        #[wf(len(prefixed(format = Ne)))]
        name: &'a str,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct Outer<'a> {
        #[wf(len(prefixed(format = Ne)))]
        inner: Inner<'a>,
        #[wf(format = Be)]
        extra: u16,
    }

    #[test]
    fn test_nested_basic() {
        let mut data = [0u8; 128];
        let msg = Outer {
            inner: Inner {
                id: 42,
                name: "test",
            },
            extra: 0xCAFE,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod header_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|_:7"))]
    struct OptU8 {
        #[wf(opt(flag = A), format = Be)]
        v: Option<u32>,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|B|C|D|_:4"))]
    struct ManyOpts {
        #[wf(opt(flag = A), format = Be)]
        a: Option<u8>,
        #[wf(opt(flag = B), format = Be)]
        b: Option<u16>,
        #[wf(opt(flag = C), format = Be)]
        c: Option<u32>,
        #[wf(opt(flag = D), format = Be)]
        d: Option<u64>,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|_:15"))]
    struct Header16 {
        #[wf(opt(flag = A), format = Be)]
        v: Option<u32>,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|_:31"))]
    struct Header32 {
        #[wf(opt(flag = A), format = Be)]
        v: Option<u32>,
    }

    #[test]
    fn test_opt_present() {
        let mut data = [0u8; 64];
        let msg = OptU8 { v: Some(42) };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_opt_absent() {
        let mut data = [0u8; 64];
        let msg = OptU8 { v: None };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_many_opts_all_present() {
        let mut data = [0u8; 64];
        let msg = ManyOpts {
            a: Some(1),
            b: Some(2),
            c: Some(3),
            d: Some(4),
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_many_opts_all_absent() {
        let mut data = [0u8; 64];
        let msg = ManyOpts {
            a: None,
            b: None,
            c: None,
            d: None,
        };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_many_opts_mixed() {
        let mut data = [0u8; 64];
        let msg = ManyOpts {
            a: Some(1),
            b: None,
            c: Some(3),
            d: None,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_header_16bits() {
        let mut data = [0u8; 64];
        let msg = Header16 { v: Some(123) };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_header_32bits() {
        let mut data = [0u8; 64];
        let msg = Header32 { v: Some(456) };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod conditional_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|_:7"))]
    struct CondScalar {
        #[wf(opt(if = 0, flag = A), format = Be)]
        v: u32,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|_:7"))]
    struct CondStr<'a> {
        #[wf(opt(if = { "default" }, flag = A), len(prefixed(format = Be)))]
        s: &'a str,
    }

    #[test]
    fn test_cond_default_omits() {
        let mut data = [0u8; 64];
        let msg = CondScalar { v: 0 };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_cond_non_default_emits() {
        let mut data = [0u8; 64];
        let msg = CondScalar { v: 42 };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_cond_str_default() {
        let mut data = [0u8; 64];
        let msg = CondStr { s: "default" };
        let (value, len) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_cond_str_other() {
        let mut data = [0u8; 64];
        let msg = CondStr { s: "other" };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod full_example {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:16"))]
    struct Attachment<'a> {
        #[wf(len(slot = S))]
        content: &'a str,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A|B|C|D|E|F|_:2"))]
    struct Message<'a> {
        #[wf(opt(flag = A), format = VLE)]
        pub opt_scalar: Option<u32>,
        #[wf(opt(flag = B), len(prefixed(format = Ne)))]
        pub opt_slice: Option<&'a str>,
        #[wf(opt(flag = C), len(prefixed(format = Ne)))]
        pub opt_msg: Option<Attachment<'a>>,

        #[wf(format = Be)]
        pub scalar: u64,
        #[wf(len(prefixed(format = Ne)))]
        pub slice: &'a [u8; 10],
        #[wf(len(prefixed(format = Le)))]
        pub msg: Attachment<'a>,

        #[wf(opt(if = 0, flag = D), format = Ne)]
        pub cond_scalar: u16,
        #[wf(opt(if = { "hello" }, flag = E), len(prefixed(format = Le)))]
        pub cond_slice: &'a str,
        #[wf(opt(if = Attachment { content: "hello" }, flag = F), len(prefixed(format = VLE)))]
        pub cond_msg: Attachment<'a>,
    }

    #[test]
    fn test_full_example_all_present() {
        let msg = Message {
            opt_scalar: Some(2),
            opt_slice: Some("AHAH"),
            opt_msg: Some(Attachment { content: "hello" }),
            scalar: 1208420121,
            slice: &[19, 29, 39, 49, 59, 69, 79, 89, 99, 109],
            msg: Attachment { content: "ICI" },
            cond_scalar: 12,
            cond_slice: "WORLD",
            cond_msg: Attachment {
                content: "biiiiiiiz",
            },
        };

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Message, elvwf::scalar::VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Message, elvwf::scalar::VLE>(buf).unwrap();
        assert_eq!(msg, value);
    }

    #[test]
    fn test_full_example_all_defaults() {
        let msg = Message {
            opt_scalar: None,
            opt_slice: None,
            opt_msg: None,
            scalar: 0,
            slice: &[0; 10],
            msg: Attachment { content: "" },
            cond_scalar: 0,
            cond_slice: "hello",
            cond_msg: Attachment { content: "hello" },
        };

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Message, elvwf::scalar::VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Message, elvwf::scalar::VLE>(buf).unwrap();
        assert_eq!(msg, value);
    }

    #[test]
    fn test_full_example_minimal() {
        let msg = Message {
            opt_scalar: None,
            opt_slice: None,
            opt_msg: None,
            scalar: 42,
            slice: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            msg: Attachment { content: "x" },
            cond_scalar: 0,
            cond_slice: "hello",
            cond_msg: Attachment { content: "hello" },
        };

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        let before = buf.len();
        elvwf::msg::encode::<Message, elvwf::scalar::VLE>(buf, msg.clone()).unwrap();
        let encoded_len = before - buf.len();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Message, elvwf::scalar::VLE>(buf).unwrap();
        assert_eq!(msg, value);
        println!("Taille minimale: {} octets", encoded_len);
    }
}

mod slot_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:8"))]
    struct Slot8<'a> {
        #[wf(len(slot = S))]
        content: &'a str,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:16"))]
    struct Slot16<'a> {
        #[wf(len(slot = S))]
        content: &'a [u8],
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:4|_:4"))]
    struct Slot4<'a> {
        #[wf(len(slot = S))]
        content: &'a [u8],
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:3|_:5"))]
    struct Slot3<'a> {
        #[wf(len(slot = S))]
        content: &'a [u8],
    }

    #[test]
    fn test_slot_8bits() {
        let mut data = [0u8; 64];
        let msg = Slot8 { content: "hi" };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_16bits() {
        let mut data = [0u8; 1024];
        let msg = Slot16 {
            content: &[42; 300],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_empty() {
        let mut data = [0u8; 64];
        let msg = Slot8 { content: "" };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_8bits_max() {
        let mut data = [0u8; 512];
        let msg = Slot8 {
            content: std::str::from_utf8(&[b'a'; 255]).unwrap(),
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_4bits_boundary() {
        let mut data = [0u8; 64];

        let msg = Slot4 { content: &[7; 15] };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_3bits_all_sizes() {
        let mut data = [0u8; 64];
        for size in 0..=7 {
            let buf: Vec<u8> = vec![0xAB; size];
            let msg = Slot3 { content: &buf };
            let (value, _) = roundtrip(msg.clone(), &mut data);
            assert_eq!(msg, value);
        }
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A:8|B:8"))]
    struct TwoSlots<'a> {
        #[wf(len(slot = A))]
        first: &'a str,
        #[wf(len(slot = B))]
        second: &'a [u8],
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A:4|B:4|C:8"))]
    struct ThreeSlots<'a> {
        #[wf(len(slot = A))]
        a: &'a [u8],
        #[wf(len(slot = B))]
        b: &'a [u8],
        #[wf(len(slot = C))]
        c: &'a str,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "X:16|Y:16"))]
    struct LargeSlots<'a> {
        #[wf(len(slot = X))]
        x: &'a [u8],
        #[wf(len(slot = Y))]
        y: &'a [u8],
    }

    #[test]
    fn test_two_slots() {
        let mut data = [0u8; 128];
        let msg = TwoSlots {
            first: "hello",
            second: &[1, 2, 3, 4, 5, 6, 7, 8],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_two_slots_both_empty() {
        let mut data = [0u8; 64];
        let msg = TwoSlots {
            first: "",
            second: &[],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_two_slots_asymmetric() {
        let mut data = [0u8; 128];
        let msg = TwoSlots {
            first: "",
            second: &[42; 100],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);

        let msg = TwoSlots {
            first: "longer string here",
            second: &[],
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_three_slots() {
        let mut data = [0u8; 128];
        let msg = ThreeSlots {
            a: &[1, 2, 3],
            b: &[4, 5, 6, 7, 8],
            c: "trois slots ici",
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_three_slots_boundaries() {
        let mut data = [0u8; 512];
        let a_data = [0xAA; 15];
        let b_data = [0xBB; 15];
        let c_data = vec![b'c'; 255];
        let msg = ThreeSlots {
            a: &a_data,
            b: &b_data,
            c: std::str::from_utf8(&c_data).unwrap(),
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_large_slots() {
        let mut data = [0u8; 4096];
        let x_buf = vec![0x55; 1000];
        let y_buf = vec![0xAA; 2000];
        let msg = LargeSlots {
            x: &x_buf,
            y: &y_buf,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:8"))]
    struct SlotWithScalars<'a> {
        #[wf(format = Be)]
        before: u32,
        #[wf(len(slot = S))]
        content: &'a [u8],
        #[wf(format = Le)]
        after: u16,
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "A:8|F|_:7"))]
    struct SlotWithFlag<'a> {
        #[wf(len(slot = A))]
        content: &'a [u8],
        #[wf(opt(flag = F), format = Be)]
        opt_field: Option<u32>,
    }

    #[test]
    fn test_slot_with_scalars() {
        let mut data = [0u8; 128];
        let msg = SlotWithScalars {
            before: 0xDEADBEEF,
            content: &[1, 2, 3, 4, 5],
            after: 0xCAFE,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_with_flag_present() {
        let mut data = [0u8; 128];
        let msg = SlotWithFlag {
            content: &[1, 2, 3],
            opt_field: Some(42),
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_with_flag_absent() {
        let mut data = [0u8; 128];
        let msg = SlotWithFlag {
            content: &[1, 2, 3],
            opt_field: None,
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:8"))]
    struct InnerSlot<'a> {
        #[wf(len(slot = S))]
        data: &'a [u8],
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    #[wf(header(dsl = "S:16"))]
    struct OuterSlot<'a> {
        #[wf(len(slot = S))]
        outer_data: &'a [u8],
        #[wf(len(prefixed(format = Be)))]
        inner: InnerSlot<'a>,
    }

    #[test]
    fn test_nested_slots() {
        let mut data = [0u8; 256];
        let msg = OuterSlot {
            outer_data: &[1; 50],
            inner: InnerSlot { data: &[2; 30] },
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }

    #[test]
    fn test_slot_8bits_overflow() {
        let mut data = [0u8; 512];
        let content_buf = vec![b'a'; 256];
        let msg = Slot8 {
            content: std::str::from_utf8(&content_buf).unwrap(),
        };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<Slot8, elvwf::scalar::VLE>(buf, msg);
        assert!(
            result.is_err(),
            "encoding 256 bytes dans un slot 8-bit devrait échouer"
        );
    }

    #[test]
    fn test_slot_4bits_overflow() {
        let mut data = [0u8; 64];
        let content = [0u8; 16];
        let msg = Slot4 { content: &content };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<Slot4, elvwf::scalar::VLE>(buf, msg);
        assert!(
            result.is_err(),
            "encoding 16 bytes dans un slot 4-bit devrait échouer"
        );
    }

    #[test]
    fn test_slot_3bits_overflow() {
        let mut data = [0u8; 64];
        let content = [0u8; 8];
        let msg = Slot3 { content: &content };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<Slot3, elvwf::scalar::VLE>(buf, msg);
        assert!(
            result.is_err(),
            "encoding 8 bytes dans un slot 3-bit devrait échouer"
        );
    }

    #[test]
    fn test_slot_16bits_overflow() {
        let mut data = vec![0u8; 70_000];
        let content = vec![0u8; 65_536];
        let msg = Slot16 { content: &content };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<Slot16, elvwf::scalar::VLE>(buf, msg);
        assert!(
            result.is_err(),
            "encoding 65536 bytes dans un slot 16-bit devrait échouer"
        );
    }

    #[test]
    fn test_multi_slot_one_overflows() {
        let mut data = [0u8; 512];
        let content_buf = vec![b'x'; 256];
        let msg = TwoSlots {
            first: std::str::from_utf8(&content_buf).unwrap(),
            second: &[1, 2, 3],
        };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<TwoSlots, elvwf::scalar::VLE>(buf, msg);
        assert!(result.is_err());

        let big = vec![0u8; 256];
        let msg = TwoSlots {
            first: "ok",
            second: &big,
        };
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<TwoSlots, elvwf::scalar::VLE>(buf, msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_slot_at_boundary_no_overflow() {
        let mut data = [0u8; 512];

        let content_buf = vec![b'a'; 255];
        let msg = Slot8 {
            content: std::str::from_utf8(&content_buf).unwrap(),
        };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);

        let content = [0u8; 15];
        let msg = Slot4 { content: &content };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);

        let content = [0u8; 7];
        let msg = Slot3 { content: &content };
        let (value, _) = roundtrip(msg.clone(), &mut data);
        assert_eq!(msg, value);
    }
}

mod error_tests {
    use super::*;

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct BigMsg {
        #[wf(format = Be)]
        v: u64,
    }

    #[test]
    fn test_encode_buffer_too_small() {
        let mut data = [0u8; 4];
        let buf = &mut &mut data[..];
        let result = elvwf::msg::encode::<BigMsg, elvwf::scalar::VLE>(buf, BigMsg { v: 42 });
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_buffer_too_small() {
        let data = [0u8; 4];
        let buf = &mut &data[..];
        let result = elvwf::msg::decode::<BigMsg, elvwf::scalar::VLE>(buf);
        assert!(result.is_err());
    }

    #[derive(Wired, Clone, Debug, PartialEq)]
    struct StrMsg<'a> {
        #[wf(len(prefixed(format = Be)))]
        s: &'a str,
    }

    #[test]
    fn test_decode_truncated_str() {
        let data = [0, 0, 0, 100, b'h', b'i'];
        let buf = &mut &data[..];
        let result = elvwf::msg::decode::<StrMsg, elvwf::scalar::VLE>(buf);
        assert!(result.is_err());
    }
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "_:8"))]
struct BasicRemaining<'a> {
    #[wf(format = Be)]
    v: u8,
    #[wf(len(remaining))]
    p: &'a [u8],
}

#[test]
fn basic_remaining_non_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        BasicRemaining {
            v: 1,
            p: &[2, 3, 4],
        },
        &mut buf,
    );
}

#[test]
fn basic_remaining_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(BasicRemaining { v: 42, p: &[] }, &mut buf);
}

#[test]
fn basic_remaining_single_byte() {
    let mut buf = [0u8; 64];
    assert_roundtrip(BasicRemaining { v: 0, p: &[0xFF] }, &mut buf);
}

#[test]
fn basic_remaining_large() {
    let big: Vec<u8> = (0..200).map(|i| i as u8).collect();
    let mut buf = [0u8; 256];
    assert_roundtrip(BasicRemaining { v: 7, p: &big }, &mut buf);
}

#[test]
fn basic_remaining_encoded_size() {
    let mut buf = [0u8; 64];
    let msg = BasicRemaining {
        v: 1,
        p: &[2, 3, 4],
    };
    let (_, len) = roundtrip(msg, &mut buf);
    assert_eq!(len, 5);
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "_:8"))]
struct RemainingStr<'a> {
    #[wf(format = Be)]
    code: u16,
    #[wf(len(remaining))]
    text: &'a str,
}

#[test]
fn remaining_str_basic() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        RemainingStr {
            code: 0xABCD,
            text: "Hello, world!",
        },
        &mut buf,
    );
}

#[test]
fn remaining_str_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(RemainingStr { code: 0, text: "" }, &mut buf);
}

#[test]
fn remaining_str_unicode() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        RemainingStr {
            code: 1,
            text: "héllo 🌍 café",
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "_:8"))]
struct MultiPrefix<'a> {
    #[wf(format = Be)]
    a: u8,
    #[wf(format = Be)]
    b: u16,
    #[wf(format = Le)]
    c: u32,
    #[wf(len(remaining))]
    rest: &'a [u8],
}

#[test]
fn multi_prefix_remaining() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        MultiPrefix {
            a: 1,
            b: 0x0203,
            c: 0xDEADBEEF,
            rest: &[10, 20, 30, 40, 50],
        },
        &mut buf,
    );
}

#[test]
fn multi_prefix_remaining_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        MultiPrefix {
            a: 0,
            b: 0,
            c: 0,
            rest: &[],
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
struct NoHeaderRemaining<'a> {
    #[wf(format = Be)]
    tag: u8,
    #[wf(len(remaining))]
    payload: &'a [u8],
}

#[test]
fn no_header_remaining() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        NoHeaderRemaining {
            tag: 9,
            payload: &[1, 2, 3, 4, 5],
        },
        &mut buf,
    );
}

#[test]
fn no_header_remaining_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        NoHeaderRemaining {
            tag: 0,
            payload: &[],
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "_:8"))]
struct InnerRemaining<'a> {
    #[wf(format = Be)]
    kind: u8,
    #[wf(len(remaining))]
    blob: &'a [u8],
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "S:16"))]
struct OuterWithInner<'a> {
    #[wf(format = Be)]
    id: u32,
    #[wf(len(slot = S))]
    inner: InnerRemaining<'a>,
}

#[test]
fn nested_remaining_inside_prefixed_message() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        OuterWithInner {
            id: 0xCAFEBABE,
            inner: InnerRemaining {
                kind: 3,
                blob: &[100, 101, 102, 103],
            },
        },
        &mut buf,
    );
}

#[test]
fn nested_remaining_inside_prefixed_message_empty_blob() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        OuterWithInner {
            id: 1,
            inner: InnerRemaining { kind: 0, blob: &[] },
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "_:8"))]
struct L1<'a> {
    #[wf(format = Be)]
    x: u8,
    #[wf(len(remaining))]
    tail: &'a [u8],
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "S:8|_:8"))]
struct L2<'a> {
    #[wf(format = Be)]
    y: u8,
    #[wf(len(slot = S))]
    l1: L1<'a>,
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "S:16"))]
struct L3<'a> {
    #[wf(format = Be)]
    z: u16,
    #[wf(len(slot = S))]
    l2: L2<'a>,
}

#[test]
fn triple_nested_remaining() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        L3 {
            z: 0x1234,
            l2: L2 {
                y: 7,
                l1: L1 {
                    x: 9,
                    tail: &[1, 2, 3, 4, 5, 6],
                },
            },
        },
        &mut buf,
    );
}

#[test]
fn triple_nested_remaining_empty_tail() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        L3 {
            z: 0,
            l2: L2 {
                y: 0,
                l1: L1 { x: 0, tail: &[] },
            },
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "S:8|_:8"))]
struct RootWithRemainingAndInner<'a> {
    #[wf(format = Be)]
    hdr: u8,
    #[wf(len(slot = S))]
    inner: InnerRemaining<'a>,
    #[wf(len(remaining))]
    outer_tail: &'a [u8],
}

#[test]
fn remaining_at_two_levels() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        RootWithRemainingAndInner {
            hdr: 1,
            inner: InnerRemaining {
                kind: 42,
                blob: &[7, 8, 9],
            },
            outer_tail: &[100, 101, 102, 103, 104],
        },
        &mut buf,
    );
}

#[test]
fn remaining_at_two_levels_inner_empty() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        RootWithRemainingAndInner {
            hdr: 2,
            inner: InnerRemaining { kind: 0, blob: &[] },
            outer_tail: &[1, 2, 3],
        },
        &mut buf,
    );
}

#[test]
fn remaining_at_two_levels_outer_empty() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        RootWithRemainingAndInner {
            hdr: 3,
            inner: InnerRemaining {
                kind: 5,
                blob: &[1, 2, 3],
            },
            outer_tail: &[],
        },
        &mut buf,
    );
}

#[test]
fn remaining_at_two_levels_both_empty() {
    let mut buf = [0u8; 128];
    assert_roundtrip(
        RootWithRemainingAndInner {
            hdr: 0,
            inner: InnerRemaining { kind: 0, blob: &[] },
            outer_tail: &[],
        },
        &mut buf,
    );
}

#[derive(Wired, Clone, PartialEq, Debug)]
#[wf(header(dsl = "F:1|_:7"))]
struct OptionalThenRemaining<'a> {
    #[wf(opt(flag = F), format = Be)]
    maybe: Option<u32>,
    #[wf(len(remaining))]
    tail: &'a [u8],
}

#[test]
fn optional_present_then_remaining() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        OptionalThenRemaining {
            maybe: Some(0xDEADBEEF),
            tail: &[1, 2, 3],
        },
        &mut buf,
    );
}

#[test]
fn optional_absent_then_remaining() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        OptionalThenRemaining {
            maybe: None,
            tail: &[1, 2, 3, 4, 5, 6, 7, 8],
        },
        &mut buf,
    );
}

#[test]
fn optional_absent_then_remaining_empty() {
    let mut buf = [0u8; 64];
    assert_roundtrip(
        OptionalThenRemaining {
            maybe: None,
            tail: &[],
        },
        &mut buf,
    );
}

#[test]
fn encoded_len_equals_fixed_plus_remaining() {
    let mut buf = [0u8; 64];
    let payload: &[u8] = &[10, 20, 30, 40, 50, 60];
    let msg = BasicRemaining { v: 99, p: payload };
    let (_, len) = roundtrip(msg, &mut buf);
    assert_eq!(len, 1 + 1 + payload.len());
}

#[test]
fn nested_remaining_length_consistency() {
    let mut buf_inner = [0u8; 64];
    let inner = InnerRemaining {
        kind: 1,
        blob: &[1, 2, 3, 4],
    };
    let (_, inner_len) = roundtrip(inner.clone(), &mut buf_inner);

    let mut buf_outer = [0u8; 128];
    let outer = OuterWithInner {
        id: 0x11223344,
        inner,
    };
    let (_, outer_len) = roundtrip(outer, &mut buf_outer);

    assert_eq!(outer_len, 4 + 2 + inner_len);
}
