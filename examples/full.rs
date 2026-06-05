use elvwf::prelude::*;

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

    #[wf(len(remaining))]
    pub payload: &'a [u8],
}

fn main() {
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
        payload: &[0, 1, 2, 3, 4, 5],
    };

    let mut data = [0u8; 128];
    let buf = &mut &mut data[..];
    elvwf::msg::encode::<Message, elvwf::scalar::VLE>(buf, msg.clone()).unwrap();

    let buf = &mut &data[..];
    let value = elvwf::msg::decode::<Message, elvwf::scalar::VLE>(buf).unwrap();

    assert_eq!(msg, value);
}
