#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod error;
pub(crate) mod raw;

pub mod scalar;
pub mod slice;

pub mod header;
pub mod msg;

pub mod bitfields;

#[cfg(feature = "randomized")]
pub use elvwf_derive::Randomized;

pub use elvwf_derive::{Wired, WiredBlank};

#[cfg(feature = "randomized")]
pub use msg::inner::Randomized;
#[cfg(feature = "randomized")]
pub use rand;

pub use msg::{Error, inner::Wired};
pub use scalar::{Be, Le, Ne, VLE};

pub mod prelude {
    pub use crate::{
        Error, Wired, bitfields,
        header::{get, has, put, trigger},
        msg,
        scalar::{self, Be, Le, Ne, VLE},
        slice,
    };

    #[cfg(feature = "randomized")]
    pub use crate::Randomized;
    #[cfg(feature = "randomized")]
    pub use crate::rand;
}
