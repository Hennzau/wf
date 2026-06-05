use crate::{header, scalar, slice};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Scalar(#[from] scalar::Error),

    #[error(transparent)]
    Slice(#[from] slice::Error),

    #[error(transparent)]
    Header(#[from] header::Error),
}

pub fn header_len<'a, T: Wired<'a>>() -> usize {
    header::len::<T::Header>()
}

pub fn body_len<'a, T: Wired<'a>>(this: &T) -> usize {
    T::body_len(this)
}

pub fn value_len<'a, T: Wired<'a>>(this: &T) -> usize {
    header_len::<T>() + body_len::<T>(this)
}

pub fn prefix_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &T) -> usize {
    C::len(value_len::<T>(this))
}

pub fn prefixed_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &T) -> usize {
    prefix_len::<T, C>(this) + value_len::<T>(this)
}

pub fn encode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: &T,
) -> Result<(), Error> {
    C::write(buf, value_len::<T>(this)).map_err(Error::from)
}

pub fn encode_header<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: &T) -> Result<(), Error> {
    header::encode::<_, T::HeaderCodec>(buf, T::header(this)?).map_err(Error::from)
}

pub fn encode_body<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
    T::encode_body(this, buf)
}

pub fn encode_value<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
    encode_header::<T>(buf, &this)?;
    encode_body::<T>(buf, this)
}

pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: T,
) -> Result<(), Error> {
    encode_len::<T, C>(buf, &this)?;
    encode_value::<T>(buf, this)
}

pub fn decode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &[u8],
) -> Result<usize, Error> {
    C::read(buf).map_err(Error::from)
}

pub fn decode_header<'a, T: Wired<'a>>(buf: &mut &[u8]) -> Result<T::Header, Error> {
    header::decode::<T::Header, T::HeaderCodec>(buf).map_err(Error::from)
}

pub fn decode_body<'a, T: Wired<'a>>(
    buf: &mut &'a [u8],
    len: usize,
    h: T::Header,
) -> Result<T, Error> {
    T::decode_body(buf, len, h)
}

pub fn decode_value<'a, T: Wired<'a>>(buf: &mut &'a [u8], len: usize) -> Result<T, Error> {
    let h = decode_header::<T>(buf)?;
    decode_body::<T>(buf, len - core::mem::size_of::<T::Header>(), h)
}

pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(buf: &mut &'a [u8]) -> Result<T, Error> {
    let len = decode_len::<T, C>(buf)?;
    decode_value::<T>(buf, len)
}

pub mod opt {
    use super::*;

    pub fn header_len<'a, T: Wired<'a>>(this: &Option<T>) -> usize {
        if this.is_some() {
            super::header_len::<T>()
        } else {
            0
        }
    }

    pub fn body_len<'a, T: Wired<'a>>(this: &Option<T>) -> usize {
        if let Some(this) = this {
            super::body_len::<T>(this)
        } else {
            0
        }
    }

    pub fn value_len<'a, T: Wired<'a>>(this: &Option<T>) -> usize {
        if let Some(this) = this {
            super::value_len::<T>(this)
        } else {
            0
        }
    }

    pub fn prefix_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &Option<T>) -> usize {
        if let Some(this) = this {
            super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn prefixed_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &Option<T>) -> usize {
        if let Some(this) = this {
            super::prefixed_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn encode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: &Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_len::<T, C>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_header<'a, T: Wired<'a>>(
        buf: &mut &mut [u8],
        this: &Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_header::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_body<'a, T: Wired<'a>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_body::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_value<'a, T: Wired<'a>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_value::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode::<T, C>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn decode_value<'a, T: Wired<'a>>(
        buf: &mut &'a [u8],
        cond: bool,
        len: usize,
    ) -> Result<Option<T>, Error> {
        if cond {
            super::decode_value::<T>(buf, len).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &'a [u8],
        cond: bool,
    ) -> Result<Option<T>, Error> {
        if cond {
            super::decode::<T, C>(buf).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub mod cond {
    use super::*;

    pub fn header_len<'a, T: Wired<'a> + PartialEq>(this: &T, skip_if: &T) -> usize {
        if this != skip_if {
            super::header_len::<T>()
        } else {
            0
        }
    }

    pub fn body_len<'a, T: Wired<'a> + PartialEq>(this: &T, skip_if: &T) -> usize {
        if this != skip_if {
            super::body_len::<T>(this)
        } else {
            0
        }
    }

    pub fn value_len<'a, T: Wired<'a> + PartialEq>(this: &T, skip_if: &T) -> usize {
        if this != skip_if {
            super::value_len::<T>(this)
        } else {
            0
        }
    }

    pub fn prefix_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        this: &T,
        skip_if: &T,
    ) -> usize {
        if this != skip_if {
            super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn prefixed_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        this: &T,
        skip_if: &T,
    ) -> usize {
        if this != skip_if {
            super::prefixed_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn encode_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: &T,
        skip_if: &T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode_len::<T, C>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_header<'a, T: Wired<'a> + PartialEq>(
        buf: &mut &mut [u8],
        this: &T,
        skip_if: &T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode_header::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_body<'a, T: Wired<'a> + PartialEq>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: &T,
    ) -> Result<(), Error> {
        if &this != skip_if {
            super::encode_body::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode_value<'a, T: Wired<'a> + PartialEq>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: &T,
    ) -> Result<(), Error> {
        if &this != skip_if {
            super::encode_value::<T>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn encode<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode::<T, C>(buf, this)
        } else {
            Ok(())
        }
    }

    pub fn decode_value<'a, T: Wired<'a>>(
        buf: &mut &'a [u8],
        cond: bool,
        len: usize,
        skip_if: T,
    ) -> Result<T, Error> {
        if cond {
            super::decode_value::<T>(buf, len)
        } else {
            Ok(skip_if)
        }
    }

    pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &'a [u8],
        cond: bool,
        skip_if: T,
    ) -> Result<T, Error> {
        if cond {
            super::decode::<T, C>(buf)
        } else {
            Ok(skip_if)
        }
    }
}

pub(crate) use inner::Wired;
pub(crate) mod inner {
    use super::*;

    pub trait Wired<'a>: Sized {
        type Header: header::Wired;
        type HeaderCodec: scalar::Codec<Self::Header>;

        fn body_len(&self) -> usize;
        fn header(&self) -> Result<Self::Header, Error>;

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), Error>;
        fn decode_body(buf: &mut &'a [u8], len: usize, header: Self::Header)
        -> Result<Self, Error>;
    }
}
