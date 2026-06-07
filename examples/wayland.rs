use core::ffi::CStr;

use elvwf::prelude::*;

#[derive(Wired, Randomized, Debug, Clone, PartialEq)]
#[wf(struct(align(u32), header(dsl = "S:32", format(le))))]
pub struct WlString<'a>(#[wf(slice(len(slot = S), bounded(low = 0, high = 32)))] &'a CStr);

impl<'a> From<&'a CStr> for WlString<'a> {
    fn from(value: &'a CStr) -> Self {
        Self(value)
    }
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq)]
pub struct WlRegistryBindPayload<'a> {
    #[wf(scalar(format(le), bounded(low = 1, high = u32::MAX)))]
    interface_id: u32,

    #[wf(msg(len(embedded)))]
    interface_name: WlString<'a>,

    #[wf(scalar(format(le), bounded(low = 1, high = u32::MAX)))]
    interface_version: u32,

    #[wf(scalar(format(le), bounded(low = 1, high = u32::MAX)))]
    new_id: u32,
}

#[derive(Wired, Randomized, Debug, Clone, PartialEq)]
#[wf(struct(header(dsl = "S:16|OP:16=0|ID:32", format(le), offset = { 8 << 48 })))]
pub struct WlRegistryBind<'a> {
    #[wf(scalar(slot = ID, bounded(low = 1, high = u32::MAX)))]
    id: u32,

    #[wf(msg(len(slot = S)))]
    payload: WlRegistryBindPayload<'a>,
}

fn main() {
    for _ in 0..1000 {
        let mut src = [0u8; 256];
        let msg = elvwf::msg::randomized::<WlRegistryBind>(&mut &mut src[..]).unwrap();

        let mut data = [0u8; 256];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<WlRegistryBind, VLE>(buf, msg.clone()).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<WlRegistryBind, VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
