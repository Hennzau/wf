use std::{convert::Infallible, num::TryFromIntError};

use crate::scalar;

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("Content is out of bound")]
    OutOfBounds,

    #[error(transparent)]
    Scalar(#[from] scalar::Error),

    #[error(transparent)]
    Conversion(#[from] TryFromIntError),

    #[error(transparent)]
    Infaillible(#[from] Infallible),
}

pub fn zero<H: Wired>() -> H {
    H::zero()
}

pub fn len<H: Wired>() -> usize {
    core::mem::size_of::<H>()
}

pub fn trigger<H: Wired>(header: H, cond: bool, flag: H) -> H {
    if cond { header.bitor(flag) } else { header }
}

pub fn has<H: Wired>(header: H, flag: H) -> bool {
    header.bitand(flag) == flag
}

pub fn put<H: Wired, E, C: TryInto<H, Error = E>>(
    header: H,
    content: C,
    slot: H,
) -> Result<H, Error>
where
    Error: From<E>,
{
    let content = content.try_into()?;

    let shifted = content.shl(slot.trailing_zeros());
    if shifted.bitand(slot) != shifted || shifted.shr(slot.trailing_zeros()) != content {
        return Err(Error::OutOfBounds);
    }

    Ok(header.bitor(shifted))
}

pub fn get<H: Wired, E, T: TryFrom<H, Error = E>>(header: H, slot: H) -> Result<T, Error>
where
    Error: From<E>,
{
    T::try_from(header.bitand(slot).shr(slot.trailing_zeros())).map_err(Error::from)
}

pub(crate) fn encode<H: Wired, C: scalar::Codec<H>>(
    buf: &mut &mut [u8],
    h: H,
) -> Result<(), Error> {
    H::encode::<C>(h, buf)
}

pub(crate) fn decode<H: Wired, C: scalar::Codec<H>>(buf: &mut &[u8]) -> Result<H, Error> {
    H::decode::<C>(buf)
}

pub(crate) use inner::Wired;
mod inner {
    use super::*;

    pub trait Wired: scalar::Wired + Sized + Copy + PartialEq {
        fn zero() -> Self;
        fn bitor(self, other: Self) -> Self;
        fn bitand(self, other: Self) -> Self;
        fn shl(self, n: u32) -> Self;
        fn shr(self, n: u32) -> Self;
        fn trailing_zeros(self) -> u32;

        fn encode<C: scalar::Codec<Self>>(self, buf: &mut &mut [u8]) -> Result<(), Error>;
        fn decode<C: scalar::Codec<Self>>(buf: &mut &[u8]) -> Result<Self, Error>;
    }

    macro_rules! uint {
        ($($uint:ty),+) => {
            $(
                impl Wired for $uint {
                    fn zero() -> Self { 0 }
                    fn bitor(self, o: Self) -> Self { self | o }
                    fn bitand(self, o: Self) -> Self { self & o }
                    fn shl(self, n: u32) -> Self { self << n }
                    fn shr(self, n: u32) -> Self { self >> n }
                    fn trailing_zeros(self) -> u32 { <$uint>::trailing_zeros(self) }
                    fn encode<C: scalar::Codec<Self>>(self, buf: &mut &mut [u8]) -> Result<(), Error> { scalar::encode::<_, C>(buf, self).map_err(Error::from) }
                    fn decode<C: scalar::Codec<Self>>(buf: &mut &[u8]) -> Result<Self, Error> { scalar::decode::<_, C>(buf).map_err(Error::from) }
                }
            )+
        };
    }

    uint!(u8, u16, u32, u64);

    impl Wired for () {
        fn zero() -> Self {}
        fn bitor(self, _: Self) -> Self {}
        fn bitand(self, _: Self) -> Self {}
        fn shl(self, _: u32) -> Self {}
        fn shr(self, _: u32) -> Self {}

        fn trailing_zeros(self) -> u32 {
            unreachable!(
                "If the crate works properly, it should never try to do anything with the unit header"
            )
        }

        fn encode<C: scalar::Codec<Self>>(self, _: &mut &mut [u8]) -> Result<(), Error> {
            Ok(())
        }

        fn decode<C: scalar::Codec<Self>>(_: &mut &[u8]) -> Result<Self, Error> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_to_vec<H: Wired>(header: H) -> Vec<u8>
    where
        scalar::Be: scalar::Codec<H>,
    {
        let mut storage = vec![0u8; scalar::len::<_, scalar::Be>(header)];
        let mut buf: &mut [u8] = &mut storage;
        encode::<_, scalar::Be>(&mut buf, header).unwrap();
        assert!(buf.is_empty());
        storage
    }

    macro_rules! flag_tests {
        ($($name:ident: $ty:ty),* $(,)?) => {
            $(
                mod $name {
                    use super::*;

                    const A: $ty = 0b0001;
                    const B: $ty = 0b0010;
                    const C: $ty = 0b0100;

                    #[test]
                    fn trigger_sets_flag() {
                        let h: $ty = 0;
                        let h = trigger(h, true, A);
                        assert_eq!(h, A);

                        let h = trigger(h, true, B);
                        assert_eq!(h, A | B);
                    }

                    #[test]
                    fn trigger_is_idempotent() {
                        let h: $ty = A;
                        assert_eq!(trigger(h, true, A), A);
                    }

                    #[test]
                    fn has_detects_set_flag() {
                        let h: $ty = A | C;
                        assert!(has(h, A));
                        assert!(has(h, C));
                        assert!(!has(h, B));
                    }

                    #[test]
                    fn has_on_empty_header_is_false() {
                        let h: $ty = 0;
                        assert!(!has(h, A));
                        assert!(!has(h, B));
                    }
                }
            )*
        };
    }

    flag_tests! {
        flags_u8: u8,
        flags_u16: u16,
        flags_u32: u32,
        flags_u64: u64,
    }

    mod fields {
        use super::*;

        const LOW: u16 = 0x000F;
        const MID: u16 = 0x00F0;
        const HIGH: u16 = 0xFF00;

        #[test]
        fn put_places_value_in_correct_slot() {
            let h: u16 = 0;
            let h = put::<u16, _, u16>(h, 0x5, LOW).unwrap();
            assert_eq!(h, 0x0005);

            let h = put::<u16, _, u16>(h, 0xA, MID).unwrap();
            assert_eq!(h, 0x00A5);

            let h = put::<u16, _, u16>(h, 0x42, HIGH).unwrap();
            assert_eq!(h, 0x42A5);
        }

        #[test]
        fn get_extracts_value_from_slot() {
            let h: u16 = 0x42A5;
            assert_eq!(get::<u16, _, u16>(h, LOW).unwrap(), 0x5);
            assert_eq!(get::<u16, _, u16>(h, MID).unwrap(), 0xA);
            assert_eq!(get::<u16, _, u16>(h, HIGH).unwrap(), 0x42);
        }

        #[test]
        fn put_then_get_roundtrip() {
            let h: u16 = 0;
            let h = put::<u16, _, u16>(h, 0x7, LOW).unwrap();
            let h = put::<u16, _, u16>(h, 0xC, MID).unwrap();
            let h = put::<u16, _, u16>(h, 0xAB, HIGH).unwrap();

            assert_eq!(get::<u16, _, u16>(h, LOW).unwrap(), 0x7);
            assert_eq!(get::<u16, _, u16>(h, MID).unwrap(), 0xC);
            assert_eq!(get::<u16, _, u16>(h, HIGH).unwrap(), 0xAB);
        }

        #[test]
        fn put_with_narrowing_conversion() {
            let h: u16 = 0;
            let h = put::<u16, _, u8>(h, 0x42u8, HIGH).unwrap();
            assert_eq!(h, 0x4200);

            let extracted: u8 = get::<u16, _, u8>(h, HIGH).unwrap();
            assert_eq!(extracted, 0x42);
        }

        #[test]
        fn get_fails_when_value_does_not_fit_target_type() {
            let h: u16 = 0xFF00;
            let res: Result<i8, _> = get::<u16, _, i8>(h, HIGH);
            assert!(matches!(res, Err(Error::Conversion(_))));
        }

        #[test]
        fn put_fails_when_content_does_not_fit_header_type() {
            let h: u16 = 0;
            let res = put::<u16, _, u32>(h, 0x1_0000u32, LOW);
            assert!(matches!(res, Err(Error::Conversion(_))));
        }

        #[test]
        fn put_does_not_clobber_other_slots() {
            let h: u16 = 0x00A0;
            let h = put::<u16, _, u16>(h, 0x3, LOW).unwrap();
            assert_eq!(h, 0x00A3);
            assert_eq!(get::<u16, _, u16>(h, MID).unwrap(), 0xA);
            assert_eq!(get::<u16, _, u16>(h, LOW).unwrap(), 0x3);
        }
    }

    #[test]
    fn field_out_of_bounds() {
        let header: u32 = 0x00000000;
        let slot: u32 = 0x000000FF;
        let value: u16 = 0x1FF;

        let result = put(header, value, slot);
        assert!(matches!(result, Err(Error::OutOfBounds)));
    }

    #[test]
    fn flag_and_field_can_coexist() {
        const FLAG: u16 = 0x0001;
        const FIELD: u16 = 0x00F0;

        let h: u16 = 0;
        let h = trigger(h, true, FLAG);
        let h = put::<u16, _, u16>(h, 0xC, FIELD).unwrap();

        assert!(has(h, FLAG));
        assert_eq!(get::<u16, _, u16>(h, FIELD).unwrap(), 0xC);
        assert_eq!(h, 0x00C1);
    }

    #[test]
    fn encode_uses_big_endian() {
        let h: u32 = 0x1234_5678;
        let bytes = encode_to_vec(h);
        assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let h: u16 = 0xABCD;
        let bytes = encode_to_vec(h);

        let mut slice: &[u8] = &bytes;
        let decoded: u16 = decode::<_, scalar::Be>(&mut slice).unwrap();
        assert_eq!(decoded, h);
        assert!(slice.is_empty());
    }

    #[test]
    fn decode_fails_on_short_buffer() {
        let bytes = [0x12];
        let mut slice: &[u8] = &bytes;
        let res: Result<u32, _> = decode::<_, scalar::Be>(&mut slice);
        assert!(matches!(res, Err(Error::Scalar(_))));
    }

    #[test]
    fn full_header_roundtrip_via_wire() {
        const FLAG_A: u16 = 0x0001;
        const FLAG_B: u16 = 0x0002;
        const FIELD: u16 = 0xFF00;

        let h: u16 = 0;
        let h = trigger(h, true, FLAG_A);
        let h = put::<u16, _, u16>(h, 0x42, FIELD).unwrap();

        let bytes = encode_to_vec(h);
        assert_eq!(bytes, vec![0x42, 0x01]);

        let mut slice: &[u8] = &bytes;
        let decoded: u16 = decode::<_, scalar::Be>(&mut slice).unwrap();

        assert!(has(decoded, FLAG_A));
        assert!(!has(decoded, FLAG_B));
        assert_eq!(get::<u16, _, u16>(decoded, FIELD).unwrap(), 0x42);
    }
}
