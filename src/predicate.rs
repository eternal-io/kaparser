use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// Match a set of items (`char`, `u8`, `T`).
pub trait Predicate<T>: Sized {
    fn predicate(&self, item: &T) -> bool;
}

impl<T, F: Fn(&T) -> bool> Predicate<T> for F {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self(item)
    }
}
impl<T: PartialOrd> Predicate<T> for Range<T> {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}
impl<T: PartialOrd> Predicate<T> for RangeInclusive<T> {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}

impl<T: PartialOrd> Predicate<T> for RangeFrom<T> {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}
impl<T: PartialOrd> Predicate<T> for RangeTo<T> {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}
impl<T: PartialOrd> Predicate<T> for RangeToInclusive<T> {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        self.contains(item)
    }
}

impl<T> Predicate<T> for RangeFull {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        let _ = item;
        true
    }
}
impl<T> Predicate<T> for () {
    #[inline(always)]
    fn predicate(&self, item: &T) -> bool {
        let _ = item;
        false
    }
}

//------------------------------------------------------------------------------

macro_rules! impl_predicate_for_primitives {
    ( $($ty:ty),+$(,)? ) => { $(
        impl Predicate<$ty> for $ty {
            #[inline(always)]
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
    ( $( $OrdN:ident ~ $IdxN:tt )+ ) => {
        impl<T, $($OrdN: Predicate<T>),+> Predicate<T> for ($($OrdN,)+) {
            #[inline(always)]
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

macro_rules! impl_predicate_for_tuples {
    ( $GenK:ident ~ $IdxK:tt $( $GenN:ident ~ $IdxN:tt )* ) => {
        impl_predicate_for_tuples!( @                   $GenK ~ $IdxK ; $($GenN ~ $IdxN)* );
    };

    ( @ $( $GenN:ident ~ $IdxN:tt )+ ; $GenK:ident ~ $IdxK:tt $( $GenM:ident ~ $IdxM:tt )* ) => {
        impl_predicate_for_tuple!( $($GenN ~ $IdxN)+ );
        impl_predicate_for_tuples!( @ $($GenN ~ $IdxN)+ $GenK ~ $IdxK ; $($GenM ~ $IdxM)* );
    };

    ( @ $( $GenN:ident ~ $IdxN:tt )+ ; ) => {
        impl_predicate_for_tuple!( $($GenN ~ $IdxN)+ );
    };
}

impl_predicate_for_tuples! {
    P1  ~ 0
    P2  ~ 1
    P3  ~ 2
    P4  ~ 3
    P5  ~ 4
    P6  ~ 5
    P7  ~ 6
    P8  ~ 7
    P9  ~ 8
    P10 ~ 9
    P11 ~ 10
    P12 ~ 11
    P13 ~ 12
    P14 ~ 13
    P15 ~ 14
    P16 ~ 15
}

//------------------------------------------------------------------------------

macro_rules! gen_ascii_predicates {
    ( $(
      $(#[$attr:meta])*
        $desc:literal $func:ident($ch:ident) => $expr:expr
    ),* $(,)? ) => { paste::paste! { $(
        #[doc = "ASCII " $desc ".\n\n"]
      $(#[$attr])*
        #[inline(always)]
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
        #[inline(always)]
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
    "[whitespace](is_ws) with No Newline."  is_ws_nn(ch) => matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b' ),
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
        "with `XID_Start` property"     xid0(ch) => unicode_ident::is_xid_start(*ch),
        "with `XID_Continue` property"  xid1(ch) => unicode_ident::is_xid_continue(*ch),

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
