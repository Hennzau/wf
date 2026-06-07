use crate::scalar;

pub use crate::error::HeaderError as Error;

impl From<core::num::TryFromIntError> for Error {
    fn from(_: core::num::TryFromIntError) -> Self {
        Error::CouldntConvert
    }
}

impl From<core::convert::Infallible> for Error {
    fn from(_: core::convert::Infallible) -> Self {
        unreachable!()
    }
}

pub fn compose<H: Wired>(header: H, other: H) -> H {
    header.bitor(other)
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
        return Err(Error::OutOfBound);
    }

    Ok(header.bitor(shifted))
}

pub fn get<H: Wired, E, C: TryFrom<H, Error = E>>(header: H, slot: H) -> Result<C, Error>
where
    Error: From<E>,
{
    C::try_from(header.bitand(slot).shr(slot.trailing_zeros())).map_err(Error::from)
}

pub mod optional {
    use super::*;

    pub fn put<H: Wired, E, C: TryInto<H, Error = E>>(
        header: H,
        content: Option<C>,
        slot: H,
    ) -> Result<H, Error>
    where
        Error: From<E>,
    {
        if let Some(content) = content {
            let content = content.try_into()?;

            let shifted = content.shl(slot.trailing_zeros());
            if shifted.bitand(slot) != shifted || shifted.shr(slot.trailing_zeros()) != content {
                return Err(Error::OutOfBound);
            }

            Ok(header.bitor(shifted))
        } else {
            Ok(header)
        }
    }

    pub fn get<H: Wired, E, C: TryFrom<H, Error = E>>(
        header: H,
        slot: H,
        flag: bool,
    ) -> Result<Option<C>, Error>
    where
        Error: From<E>,
    {
        if flag {
            C::try_from(header.bitand(slot).shr(slot.trailing_zeros()))
                .map_err(Error::from)
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

pub mod conditional {
    use super::*;

    pub fn put<H: Wired, E, C: TryInto<H, Error = E> + PartialEq>(
        header: H,
        content: C,
        slot: H,
        skip: C,
    ) -> Result<H, Error>
    where
        Error: From<E>,
    {
        if content == skip {
            return Ok(header);
        }

        let content = content.try_into()?;

        let shifted = content.shl(slot.trailing_zeros());
        if shifted.bitand(slot) != shifted || shifted.shr(slot.trailing_zeros()) != content {
            return Err(Error::OutOfBound);
        }

        Ok(header.bitor(shifted))
    }

    pub fn get<H: Wired, E, C: TryFrom<H, Error = E>>(
        header: H,
        slot: H,
        flag: bool,
        fallback: C,
    ) -> Result<C, Error>
    where
        Error: From<E>,
    {
        if flag {
            C::try_from(header.bitand(slot).shr(slot.trailing_zeros())).map_err(Error::from)
        } else {
            Ok(fallback)
        }
    }
}

pub fn zero<H: Wired>() -> H {
    H::zero()
}

pub fn len<H: Wired>() -> usize {
    core::mem::size_of::<H>()
}

pub fn encode<H: Wired, C: scalar::Codec<H>>(buf: &mut &mut [u8], h: H) -> Result<(), Error> {
    H::encode::<C>(h, buf)
}

pub fn decode<H: Wired, C: scalar::Codec<H>>(buf: &mut &[u8]) -> Result<H, Error> {
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
