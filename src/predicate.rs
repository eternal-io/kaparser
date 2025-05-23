use core::ops::{Range, RangeFull, RangeInclusive};

/// Match a set of items (`char`, `u8`, `T`).
pub trait Predicate<T>: Sized {
    fn predicate(&self, item: &T) -> bool;
}

impl<T, F: Fn(&T) -> bool> Predicate<T> for F {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        self(item)
    }
}
impl<T: PartialOrd> Predicate<T> for Range<T> {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}
impl<T: PartialOrd> Predicate<T> for RangeInclusive<T> {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}

impl<T> Predicate<T> for RangeFull {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        let _ = item;
        true
    }
}
impl<T> Predicate<T> for () {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        let _ = item;
        false
    }
}

//------------------------------------------------------------------------------

macro_rules! impl_predicate_for_primitives {
    ( $($ty:ty),+$(,)? ) => { $(
        impl Predicate<$ty> for $ty {
            #[inline]
            fn predicate(&self, item: &$ty) -> bool {
                *self == *item
            }
        }
    )+ };
}

impl_predicate_for_primitives! {
    bool, char,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
             f32, f64,
}

//------------------------------------------------------------------------------

macro_rules! impl_predicate_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => {
        impl<T, $($GenN: Predicate<T>),+> Predicate<T> for ($($GenN,)+) {
            #[inline]
            fn predicate(&self, item: &T) -> bool {
                impl_predicate_for_tuple!( @ self item $($IdxN),+ )
            }
        }
    };

    ( @ $self:ident $item:ident $IdxA:tt ) => {
        $self.$IdxA.predicate($item)
    };

    ( @ $self:ident $item:ident $IdxA:tt, $($IdxN:tt),* ) => {
        $self.$IdxA.predicate($item) || impl_predicate_for_tuple!( @ $self $item $($IdxN),* )
    };
}

__generate_codes! { impl_predicate_for_tuple ( P ) }

//------------------------------------------------------------------------------

macro_rules! gen_ascii_predicates {
    ( $(
      $(#[$attr:meta])*
        $desc:literal $func:ident($ch:ident) => $expr:expr
    ),* $(,)? ) => { paste::paste! { $(
        #[doc = "ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline]
        pub const fn $func($ch: &char) -> bool {
            $expr
        }
    )* } };
}

macro_rules! gen_unicode_predicates {
    ( $(
        $(#[$attr:meta])*
        $prop:literal $func:ident($ch:ident) => $expr:expr
    ),* $(,)? ) => { paste::paste! { $(
        #[doc = "Unicode character " $prop ".\n\n"]
      $(#[$attr])*
        #[inline]
        pub fn $func($ch: &char) -> bool {
            $expr
        }
    )* } };
}

gen_ascii_predicates! {
    /// U+0000 NUL ..= U+001F UNIT SEPARATOR, or U+007F DELETE.
    r"control"                              is_ctrl(ch)  => ch.is_ascii_control(),

    /// U+0021 ‘!’ ..= U+007E ‘~’.
    r"printable"                            is_print(ch) => ch.is_ascii_graphic(),

    /// - U+0021 ..= U+002F `! " # $ % & ' ( ) * + , - . /`, or
    /// - U+003A ..= U+0040 `: ; < = > ? @`, or
    /// - U+005B ..= U+0060 ``[ \ ] ^ _ ` ``, or
    /// - U+007B ..= U+007E `{ | } ~`
    r"punctuation"                          is_punct(ch) => ch.is_ascii_punctuation(),

    /// Note that this is different from [`char::is_ascii_whitespace`].
    /// This includes U+000B VERTICAL TAB.
    r"whitespace"                           is_ws(ch)    => matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b' | '\n'),
    "[whitespace](is_ws) with No Newline"   is_ws_nn(ch) => matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b' ),
    r"newline `\n`"                         is_nl(ch)    => matches!(ch, '\n'),

    r"any"                                  is_ascii(ch) => ch.is_ascii(),
    r"uppercase `[A-Z]`"                    is_upper(ch) => ch.is_ascii_uppercase(),
    r"lowercase `[a-z]`"                    is_lower(ch) => ch.is_ascii_lowercase(),
    r"alphabetic `[A-Za-z]`"                is_alpha(ch) => ch.is_ascii_alphabetic(),
    r"alphanumeric `[0-9A-Za-z]`"           is_alnum(ch) => ch.is_ascii_alphanumeric(),

    #[doc(alias = "is_digit")]
    r"decimal digit `[0-9]`"                is_dec(ch) => ch.is_ascii_digit(),
    r"hexadecimal digit `[0-9A-Fa-f]`"      is_hex(ch) => ch.is_ascii_hexdigit(),
    r"octal digit `[0-7]`"                  is_oct(ch) => matches!(ch, '0'..='7'),
    r"binary digit `[0-1]`"                 is_bin(ch) => matches!(ch, '0' | '1'),
}

pub mod unc {
    gen_unicode_predicates! {
        #[cfg(feature = "unicode-ident")] "with `XID_Start` property"     xid_start(ch) => unicode_ident::is_xid_start(*ch),
        #[cfg(feature = "unicode-ident")] "with `XID_Continue` property"  xid_conti(ch) => unicode_ident::is_xid_continue(*ch),

        "with `White_Space` property"             ws(ch) => ch.is_whitespace(),
        "with `Lowercase` property"            lower(ch) => ch.is_lowercase(),
        "with `Uppercase` property"            upper(ch) => ch.is_uppercase(),
        "with `Alphabetic` property"           alpha(ch) => ch.is_alphabetic(),
        "with `Alphabetic` property or in general numbers categories"
                                               alnum(ch) => ch.is_alphanumeric(),
        "in general numbers categories"          num(ch) => ch.is_numeric(),
        "in general control codes category"     ctrl(ch) => ch.is_control(),
    }
}
