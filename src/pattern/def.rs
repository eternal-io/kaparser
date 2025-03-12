#![allow(clippy::type_complexity)]
use crate::{
    combine::{Compound, Optional, Take0, com, opt, take0, take1},
    error::Situation,
    pattern::{Opaque, opaque},
    predicate::*,
};
use core::ops::RangeFrom;

#[inline(always)]
pub const fn line_end<'i, E: Situation>() -> Compound<'i, str, E, (Optional<'i, str, E, &'i str>, &'i str)> {
    com((opt("\r"), "\n"))
}

macro_rules! gen_string_patterns {
    ( $(
      $(#[$attr:meta])*
        $desc:literal $name:ident;
    )* ) => { paste::paste! { $(
        #[doc = "Zero or more ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline(always)]
        pub const fn [<$name 0>]<'i, E: Situation>()
           -> Opaque<'i, str, E, Take0<char, fn(&char) -> bool>>
            { opaque(take0([<is_ $name>])) }

        #[doc = "One or more ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline(always)]
        pub const fn [<$name 1>]<'i, E: Situation>()
           -> Opaque<'i, str, E, RangeFrom<fn(&char) -> bool>>
            { opaque(take1([<is_ $name>])) }
    )* } };
}

gen_string_patterns! {
    /// U+0021 ‘!’ ..= U+007E ‘~’.
    r"printable"                            print;
    /// - U+0021 ..= U+002F `! " # $ % & ' ( ) * + , - . /`, or
    /// - U+003A ..= U+0040 `: ; < = > ? @`, or
    /// - U+005B ..= U+0060 ``[ \ ] ^ _ ` ``, or
    /// - U+007B ..= U+007E `{ | } ~`
    r"punctuation"                          punct;
    /// Note that this is different from [`char::is_ascii_whitespace`].
    /// This includes U+000B VERTICAL TAB.
    r"whitespace"                           ws;

    r"any"                                  ascii;
    r"uppercase `[A-Z]`"                    upper;
    r"lowercase `[a-z]`"                    lower;
    r"alphabetic `[A-Za-z]`"                alpha;
    r"alphanumeric `[0-9A-Za-z]`"           alnum;

    #[doc(alias = "is_digit")]
    r"decimal digit `[0-9]`"                dec;
    r"hexadecimal digit `[0-9A-Fa-f]`"      hex;
    r"octal digit `[0-7]`"                  oct;
    r"binary digit `[0-1]`"                 bin;
}
