#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Payload was too big for the destination buffer")]
    PayloadTooBig,

    #[error("The source is too small to read all requested bytes")]
    SourceTooSmall,

    #[error("Requested slot exceeds the length of destination buffer")]
    RequestExceedsMaximum,

    #[error("Written too many bytes")]
    WrittenLengthExceedsRequest,
}

pub fn encode<'a>(buf: &mut &'a mut [u8], this: &[u8]) -> Result<&'a mut [u8], Error> {
    let (to_edit, remain) = if this.len() > buf.len() {
        return Err(Error::PayloadTooBig);
    } else {
        core::mem::take(buf).split_at_mut(this.len())
    };

    to_edit.copy_from_slice(this);
    *buf = remain;

    Ok(to_edit)
}

pub fn encode_with<'a>(
    buf: &mut &'a mut [u8],
    max: usize,
    this: impl FnOnce(&mut [u8]) -> usize,
) -> Result<&'a mut [u8], Error> {
    let this = if max > buf.len() {
        return Err(Error::RequestExceedsMaximum);
    } else {
        this(&mut buf[..max])
    };

    let (to_edit, remain) = if this > max {
        return Err(Error::WrittenLengthExceedsRequest);
    } else {
        core::mem::take(buf).split_at_mut(this)
    };

    *buf = remain;

    Ok(to_edit)
}

pub const fn decode<'a>(buf: &mut &'a [u8], len: usize) -> Result<&'a [u8], Error> {
    let (ret, remain) = if len > buf.len() {
        return Err(Error::SourceTooSmall);
    } else {
        buf.split_at(len)
    };

    *buf = remain;

    Ok(ret)
}

pub const fn fixed_decode<'a, const N: usize>(buf: &mut &'a [u8]) -> Result<&'a [u8; N], Error> {
    let (ret, remain) = if N > buf.len() {
        return Err(Error::SourceTooSmall);
    } else {
        buf.split_first_chunk::<N>()
            .expect("Length was just checked")
    };

    *buf = remain;

    Ok(ret)
}

pub const fn decode_single(buf: &mut &[u8]) -> Result<u8, Error> {
    let (ret, remain) = if buf.is_empty() {
        return Err(Error::SourceTooSmall);
    } else {
        buf.split_at(1)
    };

    *buf = remain;
    Ok(ret[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_codec {
        ($op:ident, $buf:ident, $($arg:expr),* => Ok($expected:expr), remain: $remain:expr) => {{
            let result = $op(&mut $buf, $($arg),*);
            assert_eq!(result, Ok($expected), "wrong value");
            assert_eq!($buf, $remain, "wrong remain");
        }};

        ($op:ident, $buf:ident, $($arg:expr),* => Err($err:expr)) => {{
            let result = $op(&mut $buf, $($arg),*);
            assert_eq!(result, Err($err));
        }};
    }

    #[test]
    fn encode_writes_payload_and_advances_buffer() {
        let mut storage = [0u8; 5];
        let mut buf: &mut [u8] = &mut storage;

        assert_codec!(encode, buf, &[1, 2, 3] => Ok(&mut [1, 2, 3][..]), remain: &[0, 0]);
        assert_codec!(encode, buf, &[4, 5]    => Ok(&mut [4, 5][..]),    remain: &[] as &[u8]);
    }

    #[test]
    fn encode_fails_when_payload_too_big() {
        let mut storage = [0u8; 2];
        let mut buf: &mut [u8] = &mut storage;

        assert_codec!(encode, buf, &[1, 2, 3] => Err(Error::PayloadTooBig));

        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn encode_with_writes_within_max() {
        let mut storage = [0u8; 6];
        let mut buf: &mut [u8] = &mut storage;

        let written = encode_with(&mut buf, 4, |dst| {
            dst[..3].copy_from_slice(&[7, 8, 9]);
            3
        })
        .unwrap();

        assert_eq!(written, &[7, 8, 9]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn encode_with_fails_when_max_exceeds_buffer() {
        let mut storage = [0u8; 2];
        let mut buf: &mut [u8] = &mut storage;

        let res = encode_with(&mut buf, 5, |_| 0);
        assert_eq!(res, Err(Error::RequestExceedsMaximum))
    }

    #[test]
    fn encode_with_fails_when_callback_writes_too_much() {
        let mut storage = [0u8; 4];
        let mut buf: &mut [u8] = &mut storage;

        let res = encode_with(&mut buf, 2, |_| 3);
        assert_eq!(res, Err(Error::WrittenLengthExceedsRequest));
    }

    #[test]
    fn decode_reads_requested_bytes() {
        let source = [10, 20, 30, 40];
        let mut buf: &[u8] = &source;

        assert_codec!(decode, buf, 2 => Ok(&[10, 20][..]), remain: &[30, 40]);
        assert_codec!(decode, buf, 2 => Ok(&[30, 40][..]), remain: &[] as &[u8]);
    }

    #[test]
    fn decode_fails_when_source_too_small() {
        let source = [1, 2];
        let mut buf: &[u8] = &source;

        assert_codec!(decode, buf, 5 => Err(Error::SourceTooSmall));
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn fixed_decode_reads_const_chunk() {
        let source = [1, 2, 3, 4, 5];
        let mut buf: &[u8] = &source;

        let chunk: &[u8; 3] = fixed_decode(&mut buf).unwrap();
        assert_eq!(chunk, &[1, 2, 3]);
        assert_eq!(buf, &[4, 5]);
    }

    #[test]
    fn fixed_decode_fails_when_source_too_small() {
        let source = [1, 2];
        let mut buf: &[u8] = &source;

        let res = fixed_decode::<4>(&mut buf);
        assert_eq!(res, Err(Error::SourceTooSmall));
    }

    #[test]
    fn decode_single_pops_one_byte() {
        let source = [42, 43];
        let mut buf: &[u8] = &source;

        assert_eq!(decode_single(&mut buf), Ok(42));
        assert_eq!(decode_single(&mut buf), Ok(43));
        assert_eq!(decode_single(&mut buf), Err(Error::SourceTooSmall));
    }
}
