pub(crate) mod raw;

pub mod scalar;
pub mod slice;

pub mod header;
pub mod msg;

pub use elvwf_derive::{Wired, WiredBlank};
pub use msg::{Error, inner::Wired};

pub mod prelude {
    pub use crate::{
        Error, Wired,
        header::{get, has, put, trigger},
        msg,
        scalar::{self, Ne},
        slice,
    };
}
