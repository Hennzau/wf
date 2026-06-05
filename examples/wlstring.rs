use std::ffi::CStr;

use elvwf::{WiredBlank, prelude::*};

#[derive(WiredBlank, Debug)]
#[wf(align32)]
#[wf(header(dsl = "S:32", format = Ne))]
pub struct WlString<'a> {
    #[wf(len(slot = S))]
    string: &'a CStr,
}

#[allow(unused_mut, unused_variables)]
impl<'a> Wired<'a> for WlString<'a> {
    type Header = ();
    type HeaderCodec = scalar::Ne;
    fn body_len(&self) -> usize {
        let mut len: usize = 0;
        len += 4; // len
        len += self.string.to_bytes_with_nul().len(); // str
        len += (4 - len % 4) % 4; // padding to u32
        len
    }
    fn header(&self) -> Result<Self::Header, elvwf::Error> {
        let mut h: Self::Header = elvwf::header::zero();
        Ok(h)
    }
    fn encode_body(self, buf: &mut &mut [u8]) -> Result<(), elvwf::Error> {
        let _bs: usize = buf.len();
        elvwf::scalar::encode::<u32, Ne>(buf, self.string.to_bytes_with_nul().len() as u32)?;
        elvwf::slice::encode_value::<&CStr>(buf, self.string)?;
        elvwf::slice::encode_value::<&[u8]>(buf, &(&[0; 4])[..(4 - (_bs - buf.len()) % 4) % 4])?;
        Ok(())
    }
    fn decode_body(buf: &mut &'a [u8], len: usize, h: Self::Header) -> Result<Self, elvwf::Error> {
        let _bs: usize = buf.len();
        let string_len = elvwf::scalar::decode::<u32, Ne>(buf)?;
        let string = elvwf::slice::decode_value::<&CStr>(buf, string_len as usize)?;
        let _ = elvwf::slice::decode_value::<&[u8]>(buf, (4 - (_bs - buf.len()) % 4) % 4 as usize)?;
        Ok(Self { string })
    }
}

fn main() {
    let msg = WlString { string: c"abcdefg" };

    let mut data = [0u8; 16];
    let buf = &mut &mut data[..];

    msg::encode_value::<WlString>(buf, msg).unwrap();
    let len = 16 - buf.len();

    let buf = &mut &data[..len];
    println!("{:?}", buf);
    let value = msg::decode_value::<WlString>(buf, 0).unwrap();

    println!("{:?}", value);
}
