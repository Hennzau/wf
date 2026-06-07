use elvwf::prelude::*;

bitfields! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Flags: u32 [Le] {
        const A = 0b00000001;
        const B = 0b00000010;
        const C = 0b00000100;

        const ABC = Self::A.bits() | Self::B.bits() | Self::C.bits();
    }
}

impl<'a> Randomized<'a> for Flags {
    fn randomized(_: &mut &'a mut [u8]) -> Result<Self, elvwf::msg::Error> {
        const CHOICES: [Flags; 4] = [Flags::A, Flags::B, Flags::C, Flags::ABC];

        let random = (0..rand::random_range(0..8))
            .map(|_| CHOICES[rand::random_range(0..CHOICES.len())])
            .fold(Flags::empty(), |acc, f| acc | f);

        Ok(random)
    }
}

fn main() {
    for _ in 0..1000 {
        let msg = elvwf::msg::randomized::<Flags>(&mut &mut [][..]).unwrap();

        let mut data = [0u8; 128];
        let buf = &mut &mut data[..];
        elvwf::msg::encode::<Flags, elvwf::VLE>(buf, msg).unwrap();

        let buf = &mut &data[..];
        let value = elvwf::msg::decode::<Flags, elvwf::VLE>(buf).unwrap();

        assert_eq!(msg, value);
    }
}
