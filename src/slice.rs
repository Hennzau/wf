use crate::{raw, scalar};

pub use crate::error::SliceError as Error;

impl From<core::str::Utf8Error> for Error {
    fn from(_: core::str::Utf8Error) -> Self {
        Error::InvalidUTF8
    }
}

impl From<core::ffi::FromBytesWithNulError> for Error {
    fn from(_: core::ffi::FromBytesWithNulError) -> Self {
        Error::NoNullByte
    }
}

pub mod noprefix {
    use super::*;

    pub fn len<'a, T: Wired<'a>>(this: T) -> usize {
        T::value_len(this)
    }

    pub fn encode<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
        T::encode_value(this, buf)
    }

    pub fn decode<'a, T: Wired<'a>>(buf: &mut &'a [u8], len: usize) -> Result<T, Error> {
        T::decode_value(buf, len)
    }
}

pub fn len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: T) -> usize {
    let len_len = if T::WITH_LEN {
        C::len(noprefix::len::<T>(this))
    } else {
        0
    };

    len_len + noprefix::len::<T>(this)
}

pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: T,
) -> Result<(), Error> {
    if T::WITH_LEN {
        C::write(buf, noprefix::len::<T>(this)).map_err(Error::from)?;
    }

    noprefix::encode::<T>(buf, this)
}

pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(buf: &mut &'a [u8]) -> Result<T, Error> {
    if T::WITH_LEN {
        let len = C::read(buf)?;
        noprefix::decode(buf, len)
    } else {
        noprefix::decode(buf, 0)
    }
}

#[cfg(feature = "randomized")]
pub fn randomized<'a, T: Wired<'a>>(
    buf: &mut &'a mut [u8],
    low: usize,
    high: usize,
) -> Result<T, Error> {
    T::randomized(buf, low, high)
}

#[elvwf_derive::wfu(
    tool(optional),
    base_module = super,
    template = Wired::<'a>,
    codec = scalar::Codec::<usize>)
]
pub mod optional {
    use super::*;

    #[elvwf_derive::wfu(
        tool(optional),
        base_module = super::super::noprefix,
        template = Wired::<'a>,
        decode_param(name = len, ty = usize))
    ]
    pub mod noprefix {
        use super::*;
    }

    #[cfg(feature = "randomized")]
    pub fn randomized<'a, T: Wired<'a>>(
        buf: &mut &'a mut [u8],
        low: usize,
        high: usize,
    ) -> Result<Option<T>, Error> {
        rand::random_bool(0.5)
            .then(|| T::randomized(buf, low, high))
            .transpose()
    }
}

#[elvwf_derive::wfu(
    tool(conditional),
    base_module = super,
    template = Wired::<'a>,
    codec = scalar::Codec::<usize>)
]
pub mod conditional {
    use super::*;

    #[elvwf_derive::wfu(
        tool(conditional),
        base_module = super::super::noprefix,
        template = Wired::<'a>,
        decode_param(name = len, ty = usize))
    ]
    pub mod noprefix {
        use super::*;
    }

    #[cfg(feature = "randomized")]
    pub fn randomized<'a, T: Wired<'a>>(
        buf: &mut &'a mut [u8],
        low: usize,
        high: usize,
        skip: T,
    ) -> Result<T, Error> {
        rand::random_bool(0.5)
            .then(|| T::randomized(buf, low, high))
            .transpose()
            .map(|r| r.unwrap_or(skip))
    }
}

pub(crate) use inner::Wired;
pub(crate) mod inner {
    use core::ffi::CStr;

    use super::*;

    pub trait Wired<'a>: Sized + Copy {
        const WITH_LEN: bool;

        fn value_len(self) -> usize;
        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error>;
        fn decode_value(buf: &mut &'a [u8], len: usize) -> Result<Self, Error>;

        #[cfg(feature = "randomized")]
        fn randomized(buf: &mut &'a mut [u8], low: usize, high: usize) -> Result<Self, Error>;
    }

    impl<'a> Wired<'a> for &'a [u8] {
        const WITH_LEN: bool = true;

        fn value_len(self) -> usize {
            self.len()
        }

        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error> {
            raw::encode(buf, self).map_err(Error::from).map(|_| ())
        }

        fn decode_value(buf: &mut &'a [u8], len: usize) -> Result<Self, Error> {
            raw::decode(buf, len).map_err(Error::from)
        }

        #[cfg(feature = "randomized")]
        fn randomized(buf: &mut &'a mut [u8], low: usize, high: usize) -> Result<Self, Error> {
            Ok(raw::encode_with(
                buf,
                rand::random_range(low..=high),
                |buf| {
                    rand::fill(buf);
                    Ok(buf.len())
                },
            )?)
        }
    }

    impl<'a, const N: usize> Wired<'a> for &'a [u8; N] {
        const WITH_LEN: bool = false;

        fn value_len(self) -> usize {
            self.len()
        }

        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error> {
            raw::encode(buf, self).map_err(Error::from).map(|_| ())
        }

        fn decode_value(buf: &mut &'a [u8], _: usize) -> Result<Self, Error> {
            raw::fixed_decode::<N>(buf).map_err(Error::from)
        }

        #[cfg(feature = "randomized")]
        fn randomized(buf: &mut &'a mut [u8], _: usize, _: usize) -> Result<Self, Error> {
            Ok(raw::fixed_encode_with::<N>(buf, |buf| {
                rand::fill(buf);
            })?)
        }
    }

    impl<'a> Wired<'a> for &'a str {
        const WITH_LEN: bool = true;

        fn value_len(self) -> usize {
            self.len()
        }

        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error> {
            raw::encode(buf, self.as_bytes())
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_value(buf: &mut &'a [u8], len: usize) -> Result<Self, Error> {
            core::str::from_utf8(raw::decode(buf, len).map_err(Error::from)?).map_err(Error::from)
        }

        #[cfg(feature = "randomized")]
        fn randomized(buf: &mut &'a mut [u8], low: usize, high: usize) -> Result<Self, Error> {
            let len = rand::random_range(low..=high);

            Ok(
                raw::encode_with(buf, len * char::MAX.len_utf8(), |mut buf| {
                    let mut bytes = 0usize;

                    for _ in 0..len {
                        bytes += raw::encode_with(&mut buf, char::MAX.len_utf8(), |buf| {
                            Ok(rand::random::<char>().encode_utf8(buf).len())
                        })?
                        .len();
                    }

                    Ok(bytes)
                })
                .map(core::str::from_utf8)??,
            )
        }
    }

    impl<'a> Wired<'a> for &'a CStr {
        const WITH_LEN: bool = true;

        fn value_len(self) -> usize {
            self.to_bytes_with_nul().len()
        }

        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error> {
            raw::encode(buf, self.to_bytes_with_nul())
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_value(buf: &mut &'a [u8], len: usize) -> Result<Self, Error> {
            core::ffi::CStr::from_bytes_with_nul(raw::decode(buf, len).map_err(Error::from)?)
                .map_err(Error::from)
        }

        #[cfg(feature = "randomized")]
        fn randomized(buf: &mut &'a mut [u8], low: usize, high: usize) -> Result<Self, Error> {
            let len = rand::random_range(low..=high);

            Ok(
                raw::encode_with(buf, len * char::MAX.len_utf8() + 1, |mut buf| {
                    let mut bytes = 0usize;

                    for _ in 0..len {
                        bytes += raw::encode_with(&mut buf, char::MAX.len_utf8(), |buf| {
                            Ok(loop {
                                let c = rand::random::<char>();
                                if c != '\0' {
                                    break c;
                                }
                            }
                            .encode_utf8(buf)
                            .len())
                        })?
                        .len();
                    }

                    buf[0] = b'\0';

                    Ok(bytes + 1)
                })
                .map(core::ffi::CStr::from_bytes_with_nul)??,
            )
        }
    }
}
