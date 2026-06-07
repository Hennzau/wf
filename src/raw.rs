pub use crate::error::RawError as Error;

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
    this: impl FnOnce(&mut [u8]) -> Result<usize, Error>,
) -> Result<&'a [u8], Error> {
    let this = if max > buf.len() {
        return Err(Error::RequestTooBig);
    } else {
        this(&mut buf[..max])?
    };

    let (to_edit, remain) = if this > max {
        return Err(Error::ResponseTooBig);
    } else {
        core::mem::take(buf).split_at_mut(this)
    };

    *buf = remain;

    Ok(to_edit)
}

#[cfg(feature = "randomized")]
pub fn fixed_encode_with<'a, const N: usize>(
    buf: &mut &'a mut [u8],
    this: impl FnOnce(&mut [u8; N]),
) -> Result<&'a [u8; N], Error> {
    if N > buf.len() {
        return Err(Error::RequestTooBig);
    }

    let (to_edit, remain) = core::mem::take(buf)
        .split_first_chunk_mut::<N>()
        .expect("len just checked");

    this(to_edit);
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
        buf.split_first_chunk::<N>().expect("len just checked")
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
