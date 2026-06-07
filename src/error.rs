use elvwf_derive::wfe;

#[wfe]
mod impls {
    #[wfe(domain = raw)]
    pub enum RawError {
        #[wfe(error = "payload was too big for the destination buffer")]
        PayloadTooBig = 0,
        #[wfe(error = "source is too small to read all requested bytes")]
        SourceTooSmall = 1,
        #[wfe(error = "requested length exceeds the length of destination buffer")]
        RequestTooBig = 2,
        #[wfe(error = "written too many bytes")]
        ResponseTooBig = 3,
    }

    #[wfe(domain = scalar, extends(raw))]
    pub enum ScalarError {
        #[wfe(error = "VLE value read is too big for target type")]
        VLETooBig = 4,
    }

    #[wfe(domain = slice, extends(scalar))]
    pub enum SliceError {
        #[wfe(error = "data provided is not a valid UTF8")]
        InvalidUTF8 = 5,
        #[wfe(error = "data provided is not null terminated")]
        NoNullByte = 6,
        #[wfe(error = "len isn't embedded for this type")]
        NotEmbedded = 7,
    }

    #[wfe(domain = header, extends(scalar))]
    pub enum HeaderError {
        #[wfe(error = "value is out of bounds")]
        OutOfBound = 8,
        #[wfe(error = "value cannot be converted to the requested type")]
        CouldntConvert = 9,
    }

    #[wfe(domain = msg, extends(slice, header))]
    pub enum Error {
        #[wfe(error = "unknown enum variant")]
        UnknownVariant = 10,
        #[wfe(error = "field out of bounds")]
        FieldOutOfBound = 11,
    }
}
