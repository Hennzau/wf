use std::num::TryFromIntError;

use crate::raw;

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("The VLE value read is too big for target type")]
    VLEConversion(#[from] TryFromIntError),

    #[error(transparent)]
    Raw(#[from] raw::Error),
}

pub struct Le;
pub struct Be;
pub struct Ne;
pub struct VLE;

pub fn len<T: Wired, C: Codec<T>>(this: T) -> usize {
    C::len(this)
}

pub fn decode<T: Wired, C: Codec<T>>(buf: &mut &[u8]) -> Result<T, Error> {
    C::read(buf)
}

pub fn encode<T: Wired, C: Codec<T>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
    C::write(buf, this)
}

pub mod opt {
    use super::*;

    pub fn len<T: Wired, C: Codec<T>>(this: Option<T>) -> usize {
        if let Some(this) = this {
            C::len(this)
        } else {
            0
        }
    }

    pub fn decode<T: Wired, C: Codec<T>>(buf: &mut &[u8], cond: bool) -> Result<Option<T>, Error> {
        if cond {
            Ok(Some(C::read(buf)?))
        } else {
            Ok(None)
        }
    }

    pub fn encode<T: Wired, C: Codec<T>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            C::write(buf, this)?;
        }

        Ok(())
    }
}

pub mod cond {
    use super::*;

    pub fn len<T: Wired + PartialEq, C: Codec<T>>(this: T, skip_if: T) -> usize {
        if this != skip_if { C::len(this) } else { 0 }
    }

    pub fn encode<T: Wired + PartialEq, C: Codec<T>>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: T,
    ) -> Result<(), Error> {
        if this != skip_if {
            C::write(buf, this)?;
        }

        Ok(())
    }

    pub fn decode<T: Wired, C: Codec<T>>(
        buf: &mut &[u8],
        cond: bool,
        skip_if: T,
    ) -> Result<T, Error> {
        if cond { Ok(C::read(buf)?) } else { Ok(skip_if) }
    }
}

pub(crate) use inner::{Codec, Wired};
pub(crate) mod inner {
    use super::*;

    pub trait Wired {}

    pub trait Codec<T>: Sized
    where
        T: Wired,
    {
        fn len(value: T) -> usize;
        fn read(buf: &mut &[u8]) -> Result<T, Error>;
        fn write(buf: &mut &mut [u8], value: T) -> Result<(), Error>;
    }

    impl VLE {
        const MAX: usize = Self::len(u64::MAX);

        const fn len(x: u64) -> usize {
            let bits = (u64::BITS - x.leading_zeros()) as usize;
            let n = (bits.saturating_sub(1) / 7) + 1;
            if n > 9 { 9 } else { n }
        }
    }

    impl Codec<u64> for VLE {
        fn len(value: u64) -> usize {
            VLE::len(value)
        }

        fn read(buf: &mut &[u8]) -> Result<u64, Error> {
            let mut v = 0;
            for i in 0..VLE::MAX {
                let b = raw::decode_single(buf)?;
                let shift = 7 * i;
                if i == VLE::MAX - 1 || (b & 0x80) == 0 {
                    v |= (b as u64) << shift;
                    return Ok(v);
                }
                v |= ((b & 0x7f) as u64) << shift;
            }
            unreachable!()
        }

        fn write(buf: &mut &mut [u8], mut value: u64) -> Result<(), Error> {
            raw::encode_with(buf, VLE::len(value), |out| {
                let mut len = 0;
                while value > 0x7f && len < VLE::MAX - 1 {
                    out[len] = (value as u8) | 0x80;
                    value >>= 7;
                    len += 1;
                }
                out[len] = value as u8;
                len + 1
            })?;

            Ok(())
        }
    }

    macro_rules! codec {
        (impl Codec<$int:ty> for $codec:ident { [$to:ident] [$from:ident] }) => {
            impl Codec<$int> for $codec {
                fn len(_: $int) -> usize {
                    core::mem::size_of::<$int>()
                }

                fn read(buf: &mut &[u8]) -> Result<$int, Error> {
                    Ok(<$int>::$from(
                        raw::decode(buf, core::mem::size_of::<$int>())?
                            .try_into()
                            .expect("Conversion from &[u8] failed even with correct size!"),
                    ))
                }

                fn write(buf: &mut &mut [u8], value: $int) -> Result<(), Error> { raw::encode(buf, &value.$to()).map_err(Error::from).map(|_| ()) }
            }
        };

        (belene: [$($v:ty)+]) => {
            $(
                codec!(impl Codec<$v> for Be { [to_be_bytes] [from_be_bytes] });
                codec!(impl Codec<$v> for Le { [to_le_bytes] [from_le_bytes] });
                codec!(impl Codec<$v> for Ne { [to_ne_bytes] [from_ne_bytes] });
            )+
        };

        (tag: [$($v:ty)+]) => {
            $(
                impl Wired for $v {}
            )+
        };

        (uints: [$($uint:ty)+], sints: [$($sint:ty)+], floats: [$($float:ty)+]) => {
            $(
                impl Codec<$uint> for VLE {
                    fn len(value: $uint) -> usize { <VLE as Codec<u64>>::len(value as u64) }
                    fn read(buf: &mut &[u8]) -> Result<$uint, Error> { <VLE as Codec<u64>>::read(buf).map(<$uint>::try_from).map(|ok| ok.map_err(Error::from)).flatten() }
                    fn write(buf: &mut &mut [u8], value: $uint) -> Result<(), Error> { <VLE as Codec<u64>>::write(buf, value as u64, ) }
                }
            )+

            codec!(belene: [$($uint)+ u64 $($sint)+ $($float)+]);
            codec!(tag: [$($uint)+ u64 $($sint)+ $($float)+]);
        };
    }

    codec!(uints: [u8 u16 u32 usize], sints: [i8 i16 i32 i64 isize], floats: [f32 f64]);

    impl Wired for () {}

    impl Codec<()> for Be {
        fn len(_: ()) -> usize {
            0
        }

        fn read(_: &mut &[u8]) -> Result<(), Error> {
            Ok(())
        }
        fn write(_: &mut &mut [u8], _: ()) -> Result<(), Error> {
            Ok(())
        }
    }

    impl Codec<()> for Le {
        fn len(_: ()) -> usize {
            0
        }
        fn read(_: &mut &[u8]) -> Result<(), Error> {
            Ok(())
        }
        fn write(_: &mut &mut [u8], _: ()) -> Result<(), Error> {
            Ok(())
        }
    }

    impl Codec<()> for Ne {
        fn len(_: ()) -> usize {
            0
        }
        fn read(_: &mut &[u8]) -> Result<(), Error> {
            Ok(())
        }
        fn write(_: &mut &mut [u8], _: ()) -> Result<(), Error> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded<T: Wired + Copy, C: Codec<T>>(value: T) -> Vec<u8> {
        let mut storage = vec![0u8; C::len(value)];
        let mut buf: &mut [u8] = &mut storage;
        C::write(&mut buf, value).unwrap();
        assert!(buf.is_empty());
        storage
    }

    fn roundtrip<T, C>(value: T)
    where
        T: Wired + Copy + core::fmt::Debug + PartialEq,
        C: Codec<T>,
    {
        let bytes = encoded::<T, C>(value);
        assert_eq!(bytes.len(), C::len(value),);

        let mut slice: &[u8] = &bytes;
        let decoded = C::read(&mut slice).unwrap();
        assert_eq!(decoded, value);
        assert!(slice.is_empty());
    }

    macro_rules! fixed_roundtrip_tests {
        ($($name:ident: $ty:ty = [$($val:expr),+ $(,)?]),* $(,)?) => {
            $(
                mod $name {
                    use super::*;

                    #[test]
                    fn be_roundtrip() {
                        for v in [$($val as $ty),+] { roundtrip::<$ty, Be>(v); }
                    }

                    #[test]
                    fn le_roundtrip() {
                        for v in [$($val as $ty),+] { roundtrip::<$ty, Le>(v); }
                    }

                    #[test]
                    fn ne_roundtrip() {
                        for v in [$($val as $ty),+] { roundtrip::<$ty, Ne>(v); }
                    }
                }
            )*
        };
    }

    fixed_roundtrip_tests! {
        t_u8:    u8    = [0, 1, 0x7F, 0xFF],
        t_u16:   u16   = [0, 1, 0x1234, u16::MAX],
        t_u32:   u32   = [0, 0xDEAD_BEEF, u32::MAX],
        t_u64:   u64   = [0, 0x0123_4567_89AB_CDEF, u64::MAX],
        t_usize: usize = [0, 42, usize::MAX],
        t_i8:    i8    = [i8::MIN, -1, 0, 1, i8::MAX],
        t_i16:   i16   = [i16::MIN, -1, 0, 1, i16::MAX],
        t_i32:   i32   = [i32::MIN, -1, 0, 1, i32::MAX],
        t_i64:   i64   = [i64::MIN, -1, 0, 1, i64::MAX],
        t_isize: isize = [isize::MIN, -1, 0, 1, isize::MAX],
        t_f32:   f32   = [0.0, 1.5, -3.25, f32::MIN, f32::MAX],
        t_f64:   f64   = [0.0, 1.5, -3.25, f64::MIN, f64::MAX],
    }

    #[test]
    fn be_and_le_produce_reversed_bytes() {
        let value: u32 = 0x1122_3344;
        let be = encoded::<u32, Be>(value);
        let le = encoded::<u32, Le>(value);

        assert_eq!(be, [0x11, 0x22, 0x33, 0x44]);
        assert_eq!(le, [0x44, 0x33, 0x22, 0x11]);
    }

    #[test]
    fn fixed_decode_fails_on_short_buffer() {
        let bytes = [0u8; 2];
        let mut slice: &[u8] = &bytes;
        let res = <Be as Codec<u32>>::read(&mut slice);
        assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
    }

    #[test]
    fn fixed_encode_fails_on_short_buffer() {
        let mut storage = [0u8; 2];
        let mut buf: &mut [u8] = &mut storage;
        let res = <Be as Codec<u32>>::write(&mut buf, 0xDEAD_BEEF);
        assert!(matches!(res, Err(Error::Raw(raw::Error::PayloadTooBig))));
    }

    #[test]
    fn vle_len_follows_7_bit_groups() {
        assert_eq!(<VLE as Codec<u64>>::len(0), 1);
        assert_eq!(<VLE as Codec<u64>>::len(0x7F), 1);
        assert_eq!(<VLE as Codec<u64>>::len(0x80), 2);
        assert_eq!(<VLE as Codec<u64>>::len(0x3FFF), 2);
        assert_eq!(<VLE as Codec<u64>>::len(0x4000), 3);
        assert_eq!(<VLE as Codec<u64>>::len(u64::MAX), 9);
    }

    #[test]
    fn vle_encodes_small_value_in_one_byte() {
        assert_eq!(encoded::<u64, VLE>(0), vec![0b0000_0000]);
        assert_eq!(encoded::<u64, VLE>(1), vec![0b0000_0001]);
        assert_eq!(encoded::<u64, VLE>(0x7F), vec![0b0111_1111]);
    }

    #[test]
    fn vle_encodes_with_continuation_bit() {
        assert_eq!(encoded::<u64, VLE>(0x80), vec![0x80, 0x01]);
        assert_eq!(encoded::<u64, VLE>(0x3FFF), vec![0xFF, 0x7F]);
    }

    #[test]
    fn vle_roundtrip_boundary_values() {
        for v in [
            0u64,
            1,
            0x7F,
            0x80,
            0x3FFF,
            0x4000,
            u32::MAX as u64,
            u64::MAX,
        ] {
            roundtrip::<u64, VLE>(v);
        }
    }

    macro_rules! vle_uint_tests {
        ($($name:ident: $ty:ty),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    for v in [0 as $ty, 1, <$ty>::MAX / 2, <$ty>::MAX] {
                        roundtrip::<$ty, VLE>(v);
                    }
                }
            )*
        };
    }

    vle_uint_tests! {
        vle_roundtrip_u8: u8,
        vle_roundtrip_u16: u16,
        vle_roundtrip_u32: u32,
        vle_roundtrip_usize: usize,
    }

    #[test]
    fn vle_decode_fails_on_empty() {
        let mut slice: &[u8] = &[];
        let res = <VLE as Codec<u64>>::read(&mut slice);
        assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
    }

    #[test]
    fn vle_decode_fails_when_value_too_big_for_target() {
        let bytes = encoded::<u64, VLE>(u64::MAX);
        let mut slice: &[u8] = &bytes;
        let res = <VLE as Codec<u8>>::read(&mut slice);
        assert!(matches!(res, Err(Error::VLEConversion(_))));
    }

    #[test]
    fn vle_encode_fails_on_short_buffer() {
        let mut storage = [0u8; 1];
        let mut buf: &mut [u8] = &mut storage;

        let res = <VLE as Codec<u64>>::write(&mut buf, 0x80);
        assert!(matches!(res, Err(Error::Raw(_))));
    }

    #[test]
    fn unit_codec_is_zero_length() {
        assert_eq!(<Be as Codec<()>>::len(()), 0);
        assert_eq!(<Le as Codec<()>>::len(()), 0);
        assert_eq!(<Ne as Codec<()>>::len(()), 0);

        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        <Be as Codec<()>>::write(&mut buf, ()).unwrap();

        let mut slice: &[u8] = &[];
        <Be as Codec<()>>::read(&mut slice).unwrap();
    }

    #[test]
    fn opt_len_some_matches_codec_len() {
        let value: u32 = 0xDEAD_BEEF;
        assert_eq!(
            opt::len::<u32, Be>(Some(value)),
            <Be as Codec<u32>>::len(value)
        );
    }

    #[test]
    fn opt_len_none_is_zero() {
        assert_eq!(opt::len::<u32, Be>(None), 0);
        assert_eq!(opt::len::<u64, Le>(None), 0);
        assert_eq!(opt::len::<u64, VLE>(None), 0);
    }

    #[test]
    fn opt_encode_some_writes_bytes() {
        let value: u32 = 0x1122_3344;
        let mut storage = vec![0u8; opt::len::<u32, Be>(Some(value))];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode::<u32, Be>(&mut buf, Some(value)).unwrap();
        assert!(buf.is_empty());
        assert_eq!(storage, [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn opt_encode_none_writes_nothing() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode::<u32, Be>(&mut buf, None).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn opt_decode_cond_true_reads_value() {
        let value: u32 = 0xCAFE_BABE;
        let bytes = encoded::<u32, Be>(value);
        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<u32, Be>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(value));
        assert!(slice.is_empty());
    }

    #[test]
    fn opt_decode_cond_false_returns_none_without_consuming() {
        let bytes = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<u32, Be>(&mut slice, false).unwrap();
        assert_eq!(decoded, None);
        assert_eq!(slice.len(), 4);
    }

    #[test]
    fn opt_roundtrip_some() {
        let value: u64 = 0x0123_4567_89AB_CDEF;
        let mut storage = vec![0u8; opt::len::<u64, Le>(Some(value))];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode::<u64, Le>(&mut buf, Some(value)).unwrap();
        assert!(buf.is_empty());

        let mut slice: &[u8] = &storage;
        let decoded = opt::decode::<u64, Le>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(value));
        assert!(slice.is_empty());
    }

    #[test]
    fn opt_roundtrip_none() {
        let storage: Vec<u8> = vec![];
        assert_eq!(opt::len::<u64, Le>(None), 0);

        let mut slice: &[u8] = &storage;
        let decoded = opt::decode::<u64, Le>(&mut slice, false).unwrap();
        assert_eq!(decoded, None);
    }

    #[test]
    fn opt_decode_fails_when_cond_true_but_buffer_empty() {
        let mut slice: &[u8] = &[];
        let res = opt::decode::<u32, Be>(&mut slice, true);
        assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
    }

    #[test]
    fn opt_vle_some_roundtrip() {
        let value: u64 = 0x4000;
        let mut storage = vec![0u8; opt::len::<u64, VLE>(Some(value))];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode::<u64, VLE>(&mut buf, Some(value)).unwrap();

        let mut slice: &[u8] = &storage;
        let decoded = opt::decode::<u64, VLE>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(value));
    }

    #[test]
    fn cond_len_skips_when_equal_to_skip_if() {
        assert_eq!(cond::len::<u32, Be>(0, 0), 0);
        assert_eq!(cond::len::<u64, Le>(42, 42), 0);
    }

    #[test]
    fn cond_len_returns_codec_len_when_different() {
        assert_eq!(
            cond::len::<u32, Be>(0xDEAD_BEEF, 0),
            <Be as Codec<u32>>::len(0xDEAD_BEEF)
        );
        assert_eq!(
            cond::len::<u64, VLE>(0x4000, 0),
            <VLE as Codec<u64>>::len(0x4000)
        );
    }

    #[test]
    fn cond_encode_writes_when_different_from_skip_if() {
        let value: u32 = 0x1122_3344;
        let mut storage = vec![0u8; cond::len::<u32, Be>(value, 0)];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u32, Be>(&mut buf, value, 0).unwrap();
        assert!(buf.is_empty());
        assert_eq!(storage, [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn cond_encode_skips_when_equal_to_skip_if() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u32, Be>(&mut buf, 7, 7).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn cond_decode_cond_true_reads_value() {
        let value: u32 = 0xCAFE_BABE;
        let bytes = encoded::<u32, Be>(value);
        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<u32, Be>(&mut slice, true, 0).unwrap();
        assert_eq!(decoded, value);
        assert!(slice.is_empty());
    }

    #[test]
    fn cond_decode_cond_false_returns_skip_if_without_consuming() {
        let bytes = [0xAA, 0xBB, 0xCC, 0xDD];
        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<u32, Be>(&mut slice, false, 0xDEAD).unwrap();
        assert_eq!(decoded, 0xDEAD);
        assert_eq!(slice.len(), 4);
    }

    #[test]
    fn cond_roundtrip_when_value_differs() {
        let value: u64 = 0x0123_4567_89AB_CDEF;
        let skip_if: u64 = 0;
        let len = cond::len::<u64, Le>(value, skip_if);
        let mut storage = vec![0u8; len];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u64, Le>(&mut buf, value, skip_if).unwrap();
        assert!(buf.is_empty());

        let mut slice: &[u8] = &storage;
        let decoded = cond::decode::<u64, Le>(&mut slice, true, skip_if).unwrap();
        assert_eq!(decoded, value);
        assert!(slice.is_empty());
    }

    #[test]
    fn cond_roundtrip_when_value_equals_skip_if() {
        let value: u64 = 42;
        let skip_if: u64 = 42;
        let len = cond::len::<u64, Le>(value, skip_if);
        assert_eq!(len, 0);

        let mut storage = vec![0u8; len];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u64, Le>(&mut buf, value, skip_if).unwrap();
        assert!(buf.is_empty());

        let mut slice: &[u8] = &storage;
        let decoded = cond::decode::<u64, Le>(&mut slice, false, skip_if).unwrap();
        assert_eq!(decoded, skip_if);
    }

    #[test]
    fn cond_decode_fails_when_cond_true_but_buffer_empty() {
        let mut slice: &[u8] = &[];
        let res = cond::decode::<u32, Be>(&mut slice, true, 0);
        assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
    }

    #[test]
    fn cond_vle_roundtrip_when_differs() {
        let value: u64 = 0x4000;
        let skip_if: u64 = 0;
        let len = cond::len::<u64, VLE>(value, skip_if);
        let mut storage = vec![0u8; len];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u64, VLE>(&mut buf, value, skip_if).unwrap();

        let mut slice: &[u8] = &storage;
        let decoded = cond::decode::<u64, VLE>(&mut slice, true, skip_if).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn cond_encode_skips_with_nonzero_skip_if() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<u32, Be>(&mut buf, 0xFFFF_FFFF, 0xFFFF_FFFF).unwrap();
        assert!(buf.is_empty());
    }
}
