pub use bitflags::bitflags as bitflags_external_macro;

#[macro_export]
macro_rules! bitfields {
    (@impl Wired $BitFlags: ident, $T: ty, $Fmt:ident) => {
        impl<'a> $crate::Wired<'a> for $BitFlags {
            type Header = $T;
            type HeaderCodec = $crate::$Fmt;

            fn body_len(&self) -> usize { 0 }
            fn header(&self) -> Result<Self::Header, $crate::Error> { Ok(self.bits()) }
            fn encode_body(self, _: &mut &mut [u8]) -> Result<(), $crate::Error> { Ok(()) }
            fn decode_body(_: &mut &'a [u8], _: usize, h: Self::Header) -> Result<Self, $crate::Error> { Ok(Self::from_bits_retain(h)) }
        }
    };

    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty [Le] {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:tt = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        $crate::bitfields::bitflags_external_macro! {
            $(#[$outer])*
            $vis struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }

            $($t)*
        }

        $crate::bitfields!(@impl Wired $BitFlags, $T, Le);
    };

    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty [Be] {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:tt = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        $crate::bitflags::bitflags! {
            $(#[$outer])*
            $vis struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }

            $($t)*
        }

        $crate::bitfields!(@impl $BitFlags, $T, Be);
    };

    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty [Ne] {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:tt = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        $crate::bitflags::bitflags! {
            $(#[$outer])*
            $vis struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }

            $($t)*
        }

        $crate::bitfields!(@impl $BitFlags, $T, Ne);
    };
}
