use std::{ffi::FromBytesWithNulError, str::Utf8Error};

use crate::{raw, scalar};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error(transparent)]
    FFIUtf8(#[from] FromBytesWithNulError),

    #[error(transparent)]
    Raw(#[from] raw::Error),

    #[error(transparent)]
    Scalar(#[from] scalar::Error),
}

pub fn value_len<'a, T: Wired<'a>>(this: T) -> usize {
    T::value_len(this)
}

pub fn prefix_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: T) -> usize {
    if T::WITH_LEN {
        C::len(value_len::<T>(this))
    } else {
        0
    }
}

pub fn prefixed_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: T) -> usize {
    prefix_len::<T, C>(this) + value_len::<T>(this)
}

pub fn encode_value<'a, T: Wired<'a>>(buf: &mut &mut [u8], this: T) -> Result<(), Error> {
    T::encode_value(this, buf)
}

pub fn encode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: T,
) -> Result<(), Error> {
    if T::WITH_LEN {
        C::write(buf, value_len::<T>(this)).map_err(Error::from)
    } else {
        Ok(())
    }
}

pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &mut [u8],
    this: T,
) -> Result<(), Error> {
    encode_len::<T, C>(buf, this)?;
    encode_value::<T>(buf, this)
}

pub fn decode_value<'a, T: Wired<'a>>(buf: &mut &'a [u8], len: usize) -> Result<T, Error> {
    T::decode_value(buf, len)
}

pub fn decode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
    buf: &mut &[u8],
) -> Result<usize, Error> {
    if T::WITH_LEN {
        C::read(buf).map_err(Error::from)
    } else {
        Ok(0)
    }
}

pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(buf: &mut &'a [u8]) -> Result<T, Error> {
    let len = decode_len::<T, C>(buf)?;
    decode_value::<T>(buf, len)
}

pub mod opt {
    use super::*;

    pub fn value_len<'a, T: Wired<'a>>(this: Option<T>) -> usize {
        if let Some(this) = this {
            super::value_len::<T>(this)
        } else {
            0
        }
    }

    pub fn prefix_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: Option<T>) -> usize {
        if let Some(this) = this {
            super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn prefixed_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(this: Option<T>) -> usize {
        if let Some(this) = this {
            super::value_len::<T>(this) + super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn encode_value<'a, T: Wired<'a>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_value::<T>(buf, this)?;
        }

        Ok(())
    }

    pub fn encode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_len::<T, C>(buf, this)?;
        }

        Ok(())
    }

    pub fn encode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: Option<T>,
    ) -> Result<(), Error> {
        if let Some(this) = this {
            super::encode_len::<T, C>(buf, this)?;
            super::encode_value::<T>(buf, this)?;
        }

        Ok(())
    }

    pub fn decode_value<'a, T: Wired<'a>>(
        buf: &mut &'a [u8],
        cond: bool,
        len: usize,
    ) -> Result<Option<T>, Error> {
        if cond {
            super::decode_value::<T>(buf, len).map(Option::Some)
        } else {
            Ok(None)
        }
    }

    pub fn decode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &[u8],
        cond: bool,
    ) -> Result<usize, Error> {
        if cond {
            super::decode_len::<T, C>(buf)
        } else {
            Ok(0)
        }
    }

    pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &'a [u8],
        cond: bool,
    ) -> Result<Option<T>, Error> {
        if cond {
            let len = super::decode_len::<T, C>(buf)?;
            super::decode_value::<T>(buf, len).map(Option::Some)
        } else {
            Ok(None)
        }
    }
}

pub mod cond {
    use super::*;

    pub fn value_len<'a, T: Wired<'a> + PartialEq>(this: T, skip_if: T) -> usize {
        if this != skip_if {
            super::value_len::<T>(this)
        } else {
            0
        }
    }

    pub fn prefix_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        this: T,
        skip_if: T,
    ) -> usize {
        if this != skip_if {
            super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn prefixed_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        this: T,
        skip_if: T,
    ) -> usize {
        if this != skip_if {
            super::value_len::<T>(this) + super::prefix_len::<T, C>(this)
        } else {
            0
        }
    }

    pub fn encode_value<'a, T: Wired<'a> + PartialEq>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode_value::<T>(buf, this)?;
        }

        Ok(())
    }

    pub fn encode_len<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode_len::<T, C>(buf, this)?;
        }

        Ok(())
    }

    pub fn encode<'a, T: Wired<'a> + PartialEq, C: scalar::Codec<usize>>(
        buf: &mut &mut [u8],
        this: T,
        skip_if: T,
    ) -> Result<(), Error> {
        if this != skip_if {
            super::encode_len::<T, C>(buf, this)?;
            super::encode_value::<T>(buf, this)?;
        }

        Ok(())
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

    pub fn decode_len<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &[u8],
        cond: bool,
    ) -> Result<usize, Error> {
        if cond {
            super::decode_len::<T, C>(buf)
        } else {
            Ok(0)
        }
    }

    pub fn decode<'a, T: Wired<'a>, C: scalar::Codec<usize>>(
        buf: &mut &'a [u8],
        cond: bool,
        skip_if: T,
    ) -> Result<T, Error> {
        if cond {
            let len = super::decode_len::<T, C>(buf)?;
            super::decode_value::<T>(buf, len)
        } else {
            Ok(skip_if)
        }
    }
}

pub(crate) use inner::Wired;
pub(crate) mod inner {
    use std::ffi::CStr;

    use super::*;

    pub trait Wired<'a>: Sized + Copy {
        const WITH_LEN: bool;

        fn value_len(self) -> usize;
        fn encode_value(self, buf: &mut &mut [u8]) -> Result<(), Error>;
        fn decode_value(buf: &mut &'a [u8], len: usize) -> Result<Self, Error>;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scalar::{Be, Le, VLE};

    fn write_to<F>(size: usize, f: F) -> Vec<u8>
    where
        F: FnOnce(&mut &mut [u8]) -> Result<(), Error>,
    {
        let mut storage = vec![0u8; size];
        let mut buf: &mut [u8] = &mut storage;
        f(&mut buf).unwrap();
        assert!(buf.is_empty());
        storage
    }

    mod bytes_slice {
        use super::*;

        const PAYLOAD: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];

        #[test]
        fn value_len_matches_slice_len() {
            assert_eq!(value_len::<&[u8]>(PAYLOAD), 4);
            assert_eq!(value_len::<&[u8]>(&[]), 0);
        }

        #[test]
        fn prefix_len_matches_scalar_codec() {
            assert_eq!(
                prefix_len::<&[u8], Be>(PAYLOAD),
                core::mem::size_of::<usize>()
            );

            assert_eq!(prefix_len::<&[u8], VLE>(PAYLOAD), 1);
        }

        #[test]
        fn prefixed_len_is_prefix_plus_value() {
            let expected = core::mem::size_of::<usize>() + PAYLOAD.len();
            assert_eq!(prefixed_len::<&[u8], Be>(PAYLOAD), expected);
            assert_eq!(prefixed_len::<&[u8], VLE>(PAYLOAD), 1 + PAYLOAD.len());
        }

        #[test]
        fn roundtrip_with_vle_prefix() {
            let total = prefixed_len::<&[u8], VLE>(PAYLOAD);
            let bytes = write_to(total, |buf| encode::<&[u8], VLE>(buf, PAYLOAD));

            assert_eq!(bytes[0], 4);
            assert_eq!(&bytes[1..], PAYLOAD);

            let mut slice: &[u8] = &bytes;
            let decoded = decode::<&[u8], VLE>(&mut slice).unwrap();
            assert_eq!(decoded, PAYLOAD);
            assert!(slice.is_empty());
        }

        #[test]
        fn encode_len_and_value_separately() {
            let bytes = write_to(prefixed_len::<&[u8], Be>(PAYLOAD), |buf| {
                encode_len::<&[u8], Be>(buf, PAYLOAD)?;
                encode_value::<&[u8]>(buf, PAYLOAD)
            });

            let mut slice: &[u8] = &bytes;
            let len = decode_len::<&[u8], Be>(&mut slice).unwrap();
            assert_eq!(len, PAYLOAD.len());

            let value = decode_value::<&[u8]>(&mut slice, len).unwrap();
            assert_eq!(value, PAYLOAD);
        }

        #[test]
        fn decode_value_fails_on_short_buffer() {
            let bytes = [0xAA, 0xBB];
            let mut slice: &[u8] = &bytes;
            let res = decode_value::<&[u8]>(&mut slice, 10);
            assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
        }

        #[test]
        fn empty_slice_roundtrip() {
            let empty: &[u8] = &[];
            let total = prefixed_len::<&[u8], VLE>(empty);
            let bytes = write_to(total, |buf| encode::<&[u8], VLE>(buf, empty));
            assert_eq!(bytes, vec![0]);

            let mut slice: &[u8] = &bytes;
            assert_eq!(decode::<&[u8], VLE>(&mut slice).unwrap(), empty);
        }
    }

    mod bytes_array {
        use super::*;

        const ARR: &[u8; 4] = &[1, 2, 3, 4];

        #[test]
        fn value_len_is_array_size() {
            assert_eq!(value_len::<&[u8; 4]>(ARR), 4);
        }

        #[test]
        fn prefix_len_is_always_zero() {
            assert_eq!(prefix_len::<&[u8; 4], Be>(ARR), 0);
            assert_eq!(prefix_len::<&[u8; 4], VLE>(ARR), 0);
        }

        #[test]
        fn prefixed_len_equals_value_len() {
            assert_eq!(prefixed_len::<&[u8; 4], Be>(ARR), 4);
            assert_eq!(prefixed_len::<&[u8; 4], VLE>(ARR), 4);
        }

        #[test]
        fn encode_len_is_a_noop() {
            let mut storage = [0u8; 4];
            let mut buf: &mut [u8] = &mut storage;
            encode_len::<&[u8; 4], Be>(&mut buf, ARR).unwrap();
            assert_eq!(buf.len(), 4,);
        }

        #[test]
        fn roundtrip_writes_only_payload() {
            let bytes = write_to(4, |buf| encode::<&[u8; 4], Be>(buf, ARR));
            assert_eq!(bytes, ARR);

            let mut slice: &[u8] = &bytes;
            let decoded: &[u8; 4] = decode::<&[u8; 4], Be>(&mut slice).unwrap();
            assert_eq!(decoded, ARR);
            assert!(slice.is_empty());
        }

        #[test]
        fn decode_value_ignores_len_argument() {
            let bytes = [9, 8, 7, 6];
            let mut slice: &[u8] = &bytes;
            let decoded: &[u8; 4] = decode_value::<&[u8; 4]>(&mut slice, 999).unwrap();
            assert_eq!(decoded, &[9, 8, 7, 6]);
        }

        #[test]
        fn decode_fails_on_short_buffer() {
            let bytes = [1, 2];
            let mut slice: &[u8] = &bytes;
            let res = decode::<&[u8; 4], Be>(&mut slice);
            assert!(matches!(res, Err(Error::Raw(raw::Error::SourceTooSmall))));
        }
    }

    mod string {
        use super::*;

        const HELLO: &str = "héllo";

        #[test]
        fn value_len_is_byte_length() {
            assert_eq!(value_len::<&str>(HELLO), HELLO.len());
            assert_eq!(value_len::<&str>(HELLO), 6);
        }

        #[test]
        fn roundtrip_with_vle_prefix() {
            let total = prefixed_len::<&str, VLE>(HELLO);
            let bytes = write_to(total, |buf| encode::<&str, VLE>(buf, HELLO));

            assert_eq!(bytes[0], 6);
            assert_eq!(&bytes[1..], HELLO.as_bytes());

            let mut slice: &[u8] = &bytes;
            let decoded: &str = decode::<&str, VLE>(&mut slice).unwrap();
            assert_eq!(decoded, HELLO);
            assert!(slice.is_empty());
        }

        #[test]
        fn roundtrip_with_le_prefix() {
            let total = prefixed_len::<&str, Le>(HELLO);
            let bytes = write_to(total, |buf| encode::<&str, Le>(buf, HELLO));

            let mut slice: &[u8] = &bytes;
            let decoded = decode::<&str, Le>(&mut slice).unwrap();
            assert_eq!(decoded, HELLO);
        }

        #[test]
        fn decode_fails_on_invalid_utf8() {
            let bytes = [0x02, 0xFF, 0xFF];
            let mut slice: &[u8] = &bytes;
            let res = decode::<&str, VLE>(&mut slice);
            assert!(matches!(res, Err(Error::Utf8(_))));
        }

        #[test]
        fn empty_string_roundtrip() {
            let empty = "";
            let total = prefixed_len::<&str, VLE>(empty);
            let bytes = write_to(total, |buf| encode::<&str, VLE>(buf, empty));

            let mut slice: &[u8] = &bytes;
            assert_eq!(decode::<&str, VLE>(&mut slice).unwrap(), "");
        }
    }

    #[test]
    fn slice_encoded_can_be_decoded_as_str_if_valid_utf8() {
        let original = "rust 🦀".as_bytes();
        let total = prefixed_len::<&[u8], VLE>(original);
        let bytes = write_to(total, |buf| encode::<&[u8], VLE>(buf, original));

        let mut slice: &[u8] = &bytes;
        let as_str = decode::<&str, VLE>(&mut slice).unwrap();
        assert_eq!(as_str, "rust 🦀");
    }

    const PAYLOAD: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];
    const ARR: &[u8; 4] = &[1, 2, 3, 4];
    const HELLO: &str = "héllo";
    const SKIP_ARR: &[u8; 4] = &[0, 0, 0, 0];

    #[test]
    fn opt_value_len_some_matches_inner() {
        assert_eq!(opt::value_len::<&[u8]>(Some(PAYLOAD)), PAYLOAD.len());
        assert_eq!(opt::value_len::<&[u8; 4]>(Some(ARR)), 4);
        assert_eq!(opt::value_len::<&str>(Some(HELLO)), HELLO.len());
    }

    #[test]
    fn opt_value_len_none_is_zero() {
        assert_eq!(opt::value_len::<&[u8]>(None), 0);
        assert_eq!(opt::value_len::<&[u8; 4]>(None), 0);
        assert_eq!(opt::value_len::<&str>(None), 0);
    }

    #[test]
    fn opt_prefix_len_some_matches_inner() {
        assert_eq!(
            opt::prefix_len::<&[u8], VLE>(Some(PAYLOAD)),
            prefix_len::<&[u8], VLE>(PAYLOAD)
        );
        assert_eq!(opt::prefix_len::<&[u8; 4], Be>(Some(ARR)), 0);
    }

    #[test]
    fn opt_prefix_len_none_is_zero() {
        assert_eq!(opt::prefix_len::<&[u8], Be>(None), 0);
        assert_eq!(opt::prefix_len::<&[u8], VLE>(None), 0);
        assert_eq!(opt::prefix_len::<&str, VLE>(None), 0);
    }

    #[test]
    fn opt_prefixed_len_some_matches_inner() {
        assert_eq!(
            opt::prefixed_len::<&[u8], VLE>(Some(PAYLOAD)),
            prefixed_len::<&[u8], VLE>(PAYLOAD)
        );
        assert_eq!(
            opt::prefixed_len::<&str, VLE>(Some(HELLO)),
            prefixed_len::<&str, VLE>(HELLO)
        );
    }

    #[test]
    fn opt_prefixed_len_none_is_zero() {
        assert_eq!(opt::prefixed_len::<&[u8], VLE>(None), 0);
        assert_eq!(opt::prefixed_len::<&str, VLE>(None), 0);
    }

    #[test]
    fn opt_encode_some_writes_payload() {
        let total = opt::prefixed_len::<&[u8], VLE>(Some(PAYLOAD));
        let bytes = write_to(total, |buf| opt::encode::<&[u8], VLE>(buf, Some(PAYLOAD)));
        assert_eq!(bytes[0], 4);
        assert_eq!(&bytes[1..], PAYLOAD);
    }

    #[test]
    fn opt_encode_none_writes_nothing() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode::<&[u8], VLE>(&mut buf, None).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn opt_decode_cond_true_reads_value() {
        let total = prefixed_len::<&[u8], VLE>(PAYLOAD);
        let bytes = write_to(total, |buf| encode::<&[u8], VLE>(buf, PAYLOAD));

        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<&[u8], VLE>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(PAYLOAD));
        assert!(slice.is_empty());
    }

    #[test]
    fn opt_decode_cond_false_returns_none_without_consuming() {
        let bytes = [0x04, 0xAA, 0xBB, 0xCC, 0xDD];
        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<&[u8], VLE>(&mut slice, false).unwrap();
        assert_eq!(decoded, None);
        assert_eq!(slice.len(), 5);
    }

    #[test]
    fn opt_roundtrip_some_slice() {
        let total = opt::prefixed_len::<&[u8], VLE>(Some(PAYLOAD));
        let bytes = write_to(total, |buf| opt::encode::<&[u8], VLE>(buf, Some(PAYLOAD)));

        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<&[u8], VLE>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(PAYLOAD));
        assert!(slice.is_empty());
    }

    #[test]
    fn opt_roundtrip_none_slice() {
        assert_eq!(opt::prefixed_len::<&[u8], VLE>(None), 0);
        let mut slice: &[u8] = &[];
        let decoded = opt::decode::<&[u8], VLE>(&mut slice, false).unwrap();
        assert_eq!(decoded, None);
    }

    #[test]
    fn opt_roundtrip_some_array() {
        let total = opt::prefixed_len::<&[u8; 4], Be>(Some(ARR));
        let bytes = write_to(total, |buf| opt::encode::<&[u8; 4], Be>(buf, Some(ARR)));
        assert_eq!(bytes, ARR);

        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<&[u8; 4], Be>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(ARR));
    }

    #[test]
    fn opt_roundtrip_some_str() {
        let total = opt::prefixed_len::<&str, VLE>(Some(HELLO));
        let bytes = write_to(total, |buf| opt::encode::<&str, VLE>(buf, Some(HELLO)));

        let mut slice: &[u8] = &bytes;
        let decoded = opt::decode::<&str, VLE>(&mut slice, true).unwrap();
        assert_eq!(decoded, Some(HELLO));
    }

    #[test]
    fn opt_encode_len_and_value_separately_some() {
        let total = opt::prefixed_len::<&[u8], Be>(Some(PAYLOAD));
        let bytes = write_to(total, |buf| {
            opt::encode_len::<&[u8], Be>(buf, Some(PAYLOAD))?;
            opt::encode_value::<&[u8]>(buf, Some(PAYLOAD))
        });

        let mut slice: &[u8] = &bytes;
        let len = opt::decode_len::<&[u8], Be>(&mut slice, true).unwrap();
        assert_eq!(len, PAYLOAD.len());
        let value = opt::decode_value::<&[u8]>(&mut slice, true, len).unwrap();
        assert_eq!(value, Some(PAYLOAD));
    }

    #[test]
    fn opt_encode_len_none_is_noop() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode_len::<&[u8], Be>(&mut buf, None).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn opt_encode_value_none_is_noop() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        opt::encode_value::<&[u8]>(&mut buf, None).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn opt_decode_len_cond_false_returns_zero() {
        let bytes = [0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let len = opt::decode_len::<&[u8], Be>(&mut slice, false).unwrap();
        assert_eq!(len, 0);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn opt_decode_value_cond_false_returns_none() {
        let bytes = [0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let v = opt::decode_value::<&[u8]>(&mut slice, false, 10).unwrap();
        assert_eq!(v, None);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn opt_decode_fails_when_cond_true_but_buffer_empty() {
        let mut slice: &[u8] = &[];
        let res = opt::decode::<&[u8], VLE>(&mut slice, true);
        assert!(matches!(
            res,
            Err(Error::Scalar(scalar::Error::Raw(
                raw::Error::SourceTooSmall
            )))
        ));
    }

    #[test]
    fn cond_value_len_skips_when_equal_to_skip_if() {
        let empty: &[u8] = &[];
        assert_eq!(cond::value_len::<&[u8]>(empty, empty), 0);
        assert_eq!(cond::value_len::<&[u8]>(PAYLOAD, PAYLOAD), 0);
        assert_eq!(cond::value_len::<&str>(HELLO, HELLO), 0);
    }

    #[test]
    fn cond_value_len_uses_inner_when_different() {
        let empty: &[u8] = &[];
        assert_eq!(cond::value_len::<&[u8]>(PAYLOAD, empty), PAYLOAD.len());
        assert_eq!(cond::value_len::<&str>(HELLO, ""), HELLO.len());
    }

    #[test]
    fn cond_prefix_len_skips_when_equal() {
        assert_eq!(cond::prefix_len::<&[u8], VLE>(PAYLOAD, PAYLOAD), 0);
        assert_eq!(cond::prefix_len::<&[u8], Be>(PAYLOAD, PAYLOAD), 0);
    }

    #[test]
    fn cond_prefix_len_uses_inner_when_different() {
        let empty: &[u8] = &[];
        assert_eq!(
            cond::prefix_len::<&[u8], VLE>(PAYLOAD, empty),
            prefix_len::<&[u8], VLE>(PAYLOAD)
        );
        assert_eq!(
            cond::prefix_len::<&[u8], Be>(PAYLOAD, empty),
            core::mem::size_of::<usize>()
        );
    }

    #[test]
    fn cond_prefixed_len_skips_when_equal() {
        assert_eq!(cond::prefixed_len::<&[u8], VLE>(PAYLOAD, PAYLOAD), 0);
        assert_eq!(cond::prefixed_len::<&[u8; 4], Be>(ARR, ARR), 0);
        assert_eq!(cond::prefixed_len::<&str, VLE>(HELLO, HELLO), 0);
    }

    #[test]
    fn cond_prefixed_len_matches_inner_when_different() {
        let empty: &[u8] = &[];
        assert_eq!(
            cond::prefixed_len::<&[u8], VLE>(PAYLOAD, empty),
            prefixed_len::<&[u8], VLE>(PAYLOAD)
        );
        assert_eq!(
            cond::prefixed_len::<&[u8; 4], Be>(ARR, SKIP_ARR),
            prefixed_len::<&[u8; 4], Be>(ARR)
        );
    }

    #[test]
    fn cond_encode_writes_when_different_from_skip_if() {
        let empty: &[u8] = &[];
        let total = cond::prefixed_len::<&[u8], VLE>(PAYLOAD, empty);
        let bytes = write_to(total, |buf| cond::encode::<&[u8], VLE>(buf, PAYLOAD, empty));
        assert_eq!(bytes[0], 4);
        assert_eq!(&bytes[1..], PAYLOAD);
    }

    #[test]
    fn cond_encode_skips_when_equal_to_skip_if() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<&[u8], VLE>(&mut buf, PAYLOAD, PAYLOAD).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn cond_encode_array_skips_when_equal() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode::<&[u8; 4], Be>(&mut buf, ARR, ARR).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn cond_decode_cond_true_reads_value() {
        let total = prefixed_len::<&[u8], VLE>(PAYLOAD);
        let bytes = write_to(total, |buf| encode::<&[u8], VLE>(buf, PAYLOAD));

        let empty: &[u8] = &[];
        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&[u8], VLE>(&mut slice, true, empty).unwrap();
        assert_eq!(decoded, PAYLOAD);
        assert!(slice.is_empty());
    }

    #[test]
    fn cond_decode_cond_false_returns_skip_if_without_consuming() {
        let bytes = [0x04, 0xAA, 0xBB, 0xCC, 0xDD];
        let mut slice: &[u8] = &bytes;
        let fallback: &[u8] = &[0xFF];
        let decoded = cond::decode::<&[u8], VLE>(&mut slice, false, fallback).unwrap();
        assert_eq!(decoded, fallback);
        assert_eq!(slice.len(), 5);
    }

    #[test]
    fn cond_decode_array_cond_false_returns_skip_if() {
        let bytes = [0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&[u8; 4], Be>(&mut slice, false, SKIP_ARR).unwrap();
        assert_eq!(decoded, SKIP_ARR);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn cond_roundtrip_when_value_differs() {
        let empty: &[u8] = &[];
        let total = cond::prefixed_len::<&[u8], VLE>(PAYLOAD, empty);
        let bytes = write_to(total, |buf| cond::encode::<&[u8], VLE>(buf, PAYLOAD, empty));

        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&[u8], VLE>(&mut slice, true, empty).unwrap();
        assert_eq!(decoded, PAYLOAD);
        assert!(slice.is_empty());
    }

    #[test]
    fn cond_roundtrip_when_value_equals_skip_if() {
        let total = cond::prefixed_len::<&[u8], VLE>(PAYLOAD, PAYLOAD);
        assert_eq!(total, 0);

        let bytes = write_to(total, |buf| {
            cond::encode::<&[u8], VLE>(buf, PAYLOAD, PAYLOAD)
        });
        assert!(bytes.is_empty());

        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&[u8], VLE>(&mut slice, false, PAYLOAD).unwrap();
        assert_eq!(decoded, PAYLOAD);
    }

    #[test]
    fn cond_roundtrip_str_when_differs() {
        let total = cond::prefixed_len::<&str, VLE>(HELLO, "");
        let bytes = write_to(total, |buf| cond::encode::<&str, VLE>(buf, HELLO, ""));

        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&str, VLE>(&mut slice, true, "").unwrap();
        assert_eq!(decoded, HELLO);
    }

    #[test]
    fn cond_roundtrip_str_when_equal() {
        let total = cond::prefixed_len::<&str, VLE>(HELLO, HELLO);
        assert_eq!(total, 0);

        let mut slice: &[u8] = &[];
        let decoded = cond::decode::<&str, VLE>(&mut slice, false, HELLO).unwrap();
        assert_eq!(decoded, HELLO);
    }

    #[test]
    fn cond_roundtrip_array_when_differs() {
        let total = cond::prefixed_len::<&[u8; 4], Be>(ARR, SKIP_ARR);
        let bytes = write_to(total, |buf| {
            cond::encode::<&[u8; 4], Be>(buf, ARR, SKIP_ARR)
        });
        assert_eq!(bytes, ARR);

        let mut slice: &[u8] = &bytes;
        let decoded = cond::decode::<&[u8; 4], Be>(&mut slice, true, SKIP_ARR).unwrap();
        assert_eq!(decoded, ARR);
    }

    #[test]
    fn cond_encode_len_and_value_separately_when_differs() {
        let empty: &[u8] = &[];
        let total = cond::prefixed_len::<&[u8], Be>(PAYLOAD, empty);
        let bytes = write_to(total, |buf| {
            cond::encode_len::<&[u8], Be>(buf, PAYLOAD, empty)?;
            cond::encode_value::<&[u8]>(buf, PAYLOAD, empty)
        });

        let mut slice: &[u8] = &bytes;
        let len = cond::decode_len::<&[u8], Be>(&mut slice, true).unwrap();
        assert_eq!(len, PAYLOAD.len());
        let v = cond::decode_value::<&[u8]>(&mut slice, true, len, empty).unwrap();
        assert_eq!(v, PAYLOAD);
    }

    #[test]
    fn cond_encode_len_skips_when_equal() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode_len::<&[u8], Be>(&mut buf, PAYLOAD, PAYLOAD).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn cond_encode_value_skips_when_equal() {
        let mut storage = [0u8; 0];
        let mut buf: &mut [u8] = &mut storage;
        cond::encode_value::<&[u8]>(&mut buf, PAYLOAD, PAYLOAD).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn cond_decode_len_cond_false_returns_zero() {
        let bytes = [0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let len = cond::decode_len::<&[u8], Be>(&mut slice, false).unwrap();
        assert_eq!(len, 0);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn cond_decode_value_cond_false_returns_skip_if() {
        let bytes = [0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let fallback: &[u8] = &[0xFF];
        let v = cond::decode_value::<&[u8]>(&mut slice, false, 10, fallback).unwrap();
        assert_eq!(v, fallback);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn cond_decode_fails_when_cond_true_but_buffer_empty() {
        let empty: &[u8] = &[];
        let mut slice: &[u8] = &[];
        let res = cond::decode::<&[u8], VLE>(&mut slice, true, empty);
        assert!(matches!(
            res,
            Err(Error::Scalar(scalar::Error::Raw(
                raw::Error::SourceTooSmall
            )))
        ));
    }
}
