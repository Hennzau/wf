#[cfg(feature = "randomized")]
use crate::Randomized;
use crate::{header, scalar};
use elvwf_derive::wfu;

pub use crate::error::Error;

pub mod noprefixnohead {
    use super::*;

    pub fn len<'a, T: Wired<'a>>(this: &T) -> usize {
        T::body_len(this)
    }

    pub fn encode<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
        T::encode_body(this, buf)
    }

    pub fn decode<'a, T: Wired<'a>>(
        buf: &mut &'a [u8],
        h: T::Header,
        len: usize,
    ) -> Result<T, Error> {
        T::decode_body(buf, len, h)
    }
}

pub mod noprefix {
    use super::*;

    pub fn len<'a, T: Wired<'a>>(this: &T) -> usize {
        header::len::<T::Header>() + super::noprefixnohead::len(this)
    }

    pub fn encode<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
        header::encode::<T::Header, T::HeaderCodec>(buf, T::header(&this)?)?;
        super::noprefixnohead::encode::<T>(buf, this)
    }

    pub fn decode<'a, T: Wired<'a>>(buf: &mut &'a [u8], len: usize) -> Result<T, Error> {
        let h = header::decode::<T::Header, T::HeaderCodec>(buf)?;
        super::noprefixnohead::decode(buf, h, len - core::mem::size_of::<T::Header>())
    }
}

pub mod nohead {
    use super::*;

    pub fn len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &T) -> usize {
        C::len(super::noprefixnohead::len(this)) + super::noprefixnohead::len(this)
    }

    pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: T,
    ) -> Result<(), Error> {
        C::write(buf, super::noprefixnohead::len(&this))?;
        super::noprefixnohead::encode::<T>(buf, this)
    }

    pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &'a [u8],
        h: T::Header,
    ) -> Result<T, Error> {
        let len = C::read(buf)?;
        super::noprefixnohead::decode(buf, h, len)
    }
}

pub fn len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: &T) -> usize {
    C::len(noprefix::len(this)) + noprefix::len(this)
}

pub fn header<'a, T: Wired<'a>>(this: &T) -> Result<T::Header, Error> {
    T::header(this)
}

pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: T,
) -> Result<(), Error> {
    C::write(buf, noprefix::len(&this))?;
    noprefix::encode::<T>(buf, this)
}

pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(buf: &mut &'a [u8]) -> Result<T, Error> {
    let len = C::read(buf)?;
    noprefix::decode(buf, len)
}

#[cfg(feature = "randomized")]
pub fn randomized<'a, T: Randomized<'a>>(buf: &mut &'a mut [u8]) -> Result<T, Error> {
    T::randomized(buf)
}

#[wfu(
    tool(optional),
    base_module = super,
    template = Wired::<'a>,
    codec = scalar::Codec::<usize>,
    borrow
)]
pub mod optional {
    use super::*;

    #[wfu(
        tool(optional),
        base_module = super::super::noprefixnohead,
        template = Wired::<'a>,
        decode_param(name = h, ty = T::Header),
        decode_param(name = len, ty = usize),
        borrow
    )]
    pub mod noprefixnohead {
        use super::super::*;
    }
    #[wfu(
        tool(optional),
        base_module = super::super::noprefix,
        template = Wired::<'a>,
        decode_param(name = len, ty = usize),
        borrow
    )]
    pub mod noprefix {
        use super::super::*;
    }

    #[wfu(
        tool(optional),
        base_module = super::super::nohead,
        template = Wired::<'a>,
        codec = scalar::Codec::<usize>,
        decode_param(name = h, ty = T::Header),
        borrow
    )]
    pub mod nohead {
        use super::super::*;
    }

    #[cfg(feature = "randomized")]
    pub fn randomized<'a, T: Randomized<'a>>(buf: &mut &'a mut [u8]) -> Result<Option<T>, Error> {
        rand::random_bool(0.5)
            .then(|| T::randomized(buf))
            .transpose()
    }
}

#[wfu(
    tool(conditional),
    base_module = super,
    template = Wired::<'a>,
    codec = scalar::Codec::<usize>,
    borrow
)]
pub mod conditional {
    use super::*;

    #[wfu(
        tool(conditional),
        base_module = super::super::noprefixnohead,
        template = Wired::<'a>,
        decode_param(name = h, ty = T::Header),
        decode_param(name = len, ty = usize),
        borrow
    )]
    pub mod noprefixnohead {
        use super::super::*;
    }
    #[wfu(
        tool(conditional),
        base_module = super::super::noprefix,
        template = Wired::<'a>,
        decode_param(name = len, ty = usize),
        borrow
    )]
    pub mod noprefix {
        use super::super::*;
    }

    #[wfu(
        tool(conditional),
        base_module = super::super::nohead,
        template = Wired::<'a>,
        codec = scalar::Codec::<usize>,
        decode_param(name = h, ty = T::Header),
        borrow
    )]
    pub mod nohead {
        use super::super::*;
    }

    #[cfg(feature = "randomized")]
    pub fn randomized<'a, T: Randomized<'a>>(buf: &mut &'a mut [u8], skip: T) -> Result<T, Error> {
        rand::random_bool(0.5)
            .then(|| T::randomized(buf))
            .transpose()
            .map(|r| r.unwrap_or(skip))
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
        fn decode_body(buf: &mut &'a [u8], len: usize, h: Self::Header) -> Result<Self, Error>;
    }

    #[cfg(feature = "randomized")]
    pub trait Randomized<'a>: Sized {
        fn randomized(buf: &mut &'a mut [u8]) -> Result<Self, crate::msg::Error>;
    }
}

#[cfg(feature = "alloc")]
mod impls {
    use crate::slice;
    use core::ffi::CStr;

    use super::inner::*;
    use super::*;

    impl<'a> Wired<'a> for alloc::vec::Vec<u8> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::vec::Vec::from(
                slice::noprefix::decode::<&[u8]>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a, const N: usize> Wired<'a> for [u8; N] {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8; N]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8; N]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(*slice::noprefix::decode::<&[u8; N]>(buf, len).map_err(Error::from)?)
        }
    }

    impl<'a> Wired<'a> for alloc::boxed::Box<[u8]> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::boxed::Box::from(
                slice::noprefix::decode::<&[u8]>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::rc::Rc<[u8]> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::rc::Rc::from(
                slice::noprefix::decode::<&[u8]>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::sync::Arc<[u8]> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::sync::Arc::from(
                slice::noprefix::decode::<&[u8]>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::borrow::Cow<'a, [u8]> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&[u8]>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&[u8]>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::borrow::Cow::Borrowed(
                slice::noprefix::decode::<&[u8]>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::boxed::Box<str> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&str>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&str>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::boxed::Box::from(
                slice::noprefix::decode::<&str>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::rc::Rc<str> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&str>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&str>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::rc::Rc::from(
                slice::noprefix::decode::<&str>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::sync::Arc<str> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&str>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&str>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::sync::Arc::from(
                slice::noprefix::decode::<&str>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::borrow::Cow<'a, str> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&str>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&str>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::borrow::Cow::Borrowed(
                slice::noprefix::decode::<&str>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::boxed::Box<CStr> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&CStr>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&CStr>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::boxed::Box::from(
                slice::noprefix::decode::<&CStr>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::rc::Rc<CStr> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&CStr>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&CStr>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::rc::Rc::from(
                slice::noprefix::decode::<&CStr>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::sync::Arc<CStr> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&CStr>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&CStr>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::sync::Arc::from(
                slice::noprefix::decode::<&CStr>(buf, len).map_err(Error::from)?,
            ))
        }
    }

    impl<'a> Wired<'a> for alloc::borrow::Cow<'a, CStr> {
        type Header = ();
        type HeaderCodec = super::scalar::Be;

        fn body_len(&self) -> usize {
            slice::noprefix::len::<&CStr>(self)
        }

        fn header(&self) -> Result<Self::Header, super::Error> {
            Ok(())
        }

        fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), super::Error> {
            slice::noprefix::encode::<&CStr>(buf, &self)
                .map_err(Error::from)
                .map(|_| ())
        }

        fn decode_body(buf: &mut &'a [u8], len: usize, _: Self::Header) -> Result<Self, Error> {
            Ok(alloc::borrow::Cow::Borrowed(
                slice::noprefix::decode::<&CStr>(buf, len).map_err(Error::from)?,
            ))
        }
    }
}
