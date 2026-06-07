use crate::raw;

pub use crate::error::ScalarError as Error;

impl From<core::num::TryFromIntError> for Error {
    fn from(_: core::num::TryFromIntError) -> Self {
        Error::VLETooBig
    }
}

pub struct Le;
pub struct Be;
pub struct Ne;
pub struct VLE;

pub fn len<T: Wired, C: Codec<T>>(this: T) -> usize {
    C::len(this)
}

pub fn encode<T: Wired, C: Codec<T>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
    C::write(buf, this)
}

pub fn decode<T: Wired, C: Codec<T>>(buf: &mut &[u8]) -> Result<T, Error> {
    C::read(buf)
}

#[cfg(feature = "randomized")]
pub fn randomized<T: Wired>(low: T, high: T) -> T {
    T::randomized(low, high)
}

#[elvwf_derive::wfu(
    tool(optional),
    base_module = super,
    template = Wired,
    codec = Codec::<T>)
]
pub mod optional {
    use super::*;

    #[cfg(feature = "randomized")]
    pub fn randomized<T: Wired>(low: T, high: T) -> Option<T> {
        rand::random_bool(0.5).then(|| T::randomized(low, high))
    }
}

#[elvwf_derive::wfu(
    tool(conditional),
    base_module = super,
    template = Wired,
    codec = Codec::<T>)
]
pub mod conditional {
    use super::*;

    #[cfg(feature = "randomized")]
    pub fn randomized<T: Wired>(low: T, high: T, skip: T) -> T {
        if rand::random_bool(0.5) {
            T::randomized(low, high)
        } else {
            skip
        }
    }
}

pub(crate) use inner::{Codec, Wired};
pub(crate) mod inner {
    use super::*;

    pub trait Wired {
        #[cfg(feature = "randomized")]
        fn randomized(low: Self, high: Self) -> Self;
    }

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
                Ok(len + 1)
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

        (uints: [$($uint:ty)+], sints: [$($sint:ty)+], floats: [$($float:ty)+]) => {
            $(
                impl Codec<$uint> for VLE {
                    fn len(value: $uint) -> usize { <VLE as Codec<u64>>::len(value as u64) }
                    fn read(buf: &mut &[u8]) -> Result<$uint, Error> { <VLE as Codec<u64>>::read(buf).map(<$uint>::try_from).map(|ok| ok.map_err(Error::from)).flatten() }
                    fn write(buf: &mut &mut [u8], value: $uint) -> Result<(), Error> { <VLE as Codec<u64>>::write(buf, value as u64, ) }
                }
            )+

            codec!(belene: [$($uint)+ u64 $($sint)+ $($float)+]);
        };
    }

    codec!(uints: [u8 u16 u32 usize], sints: [i8 i16 i32 i64 isize], floats: [f32 f64]);

    macro_rules! wired {
        ([$($v:ty)+]) => {
            $(
                impl Wired for $v {
                    #[cfg(feature = "randomized")]
                    fn randomized(low: $v, high: $v) -> $v {
                        rand::random_range(low..=high)
                    }
                }
            )+
        };
    }

    wired!([u8 u16 u32 u64 i8 i16 i32 i64 f32 f64]);

    impl Wired for usize {
        #[cfg(feature = "randomized")]
        fn randomized(low: Self, high: Self) -> Self {
            rand::random_range((low as u64)..(high as u64)) as usize
        }
    }

    impl Wired for isize {
        #[cfg(feature = "randomized")]
        fn randomized(low: Self, high: Self) -> Self {
            rand::random_range((low as i64)..(high as i64)) as isize
        }
    }

    impl Wired for () {
        #[cfg(feature = "randomized")]
        fn randomized(_: Self, _: Self) -> Self {}
    }

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
