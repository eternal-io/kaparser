use crate::prelude::*;

#[inline]
pub const fn line_end<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com((opt("\r"), "\n"))
}

#[inline]
pub const fn ident<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([('_', is_alpha)], take0more(is_alnum)))
}

#[inline]
#[cfg(feature = "unicode-ident")]
pub const fn unc_ident<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([('_', unc::xid_start)], take0more(unc::xid_conti)))
}

#[inline]
pub const fn hex_hex<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([is_hex], take0more(('_', is_hex))))
}
#[inline]
pub const fn dec_dec<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([is_dec], take0more(('_', is_dec))))
}
#[inline]
pub const fn oct_oct<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([is_oct], take0more(('_', is_oct))))
}
#[inline]
pub const fn bin_bin<'i, E: Situation>() -> impl Pattern<'i, str, E, Captured = &'i str> {
    com(([is_bin], take0more(('_', is_bin))))
}

macro_rules! gen_string_patterns {
    ( $(
      $(#[$attr:meta])*
        $desc:literal $name:ident;
    )* ) => { paste::paste! { $(
        #[doc = "Zero or more ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline]
        pub const fn [<$name 0>]<'i, E: Situation>()
           -> impl Pattern<'i, str, E, Captured = &'i str>
            { take0more([<is_ $name>]) }

        #[doc = "One or more ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline]
        pub const fn [<$name 1>]<'i, E: Situation>()
           -> impl Pattern<'i, str, E, Captured = &'i str>
            { take1more([<is_ $name>]) }
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
