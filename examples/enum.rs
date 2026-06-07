use elvwf::prelude::*;

#[derive(Wired, Randomized, Clone, Copy, Debug, PartialEq)]
#[wf(enum(repr(u32), format(le)))]
enum MyEnum {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
}

fn main() {
    for _ in 0..1000 {
        let msg = elvwf::msg::randomized::<MyEnum>(&mut &mut [][..]).unwrap();

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<MyEnum, elvwf::VLE>(buf, msg).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<MyEnum, elvwf::VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
