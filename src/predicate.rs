use super::*;
use core::ops::{Range, RangeFull, RangeInclusive};

/// Match a set of items (`char`, `u8`, `T`).
pub trait Predicate<T> {
    fn predicate(&self, value: &T) -> bool;
}

impl<T, F: Fn(&T) -> bool> Predicate<T> for F {
    #[inline(always)]
    fn predicate(&self, value: &T) -> bool {
        self(value)
    }
}

impl<T: PartialOrd> Predicate<T> for Range<T> {
    #[inline(always)]
    fn predicate(&self, value: &T) -> bool {
        self.contains(value)
    }
}
impl<T: PartialOrd> Predicate<T> for RangeInclusive<T> {
    #[inline(always)]
    fn predicate(&self, value: &T) -> bool {
        self.contains(value)
    }
}
impl<T> Predicate<T> for RangeFull {
    #[inline(always)]
    fn predicate(&self, value: &T) -> bool {
        let _ = value;
        true
    }
}
impl<T> Predicate<T> for () {
    #[inline(always)]
    fn predicate(&self, value: &T) -> bool {
        let _ = value;
        false
    }
}

//------------------------------------------------------------------------------

macro_rules! impl_predicate_for_primitives {
    ( $($ty:ty),+$(,)? ) => { $(
        impl Predicate<$ty> for $ty {
            #[inline(always)]
            fn predicate(&self, value: &$ty) -> bool {
                *self == *value
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
            fn predicate(&self, value: &T) -> bool {
                impl_predicate_for_tuple!( @ self value $($IdxN),+ )
            }
        }
    };

    ( @ $self:ident $value:ident $IdxA:tt ) => {
        $self.$IdxA.predicate($value)
    };

    ( @ $self:ident $value:ident $IdxA:tt, $($IdxN:tt),* ) => {
        $self.$IdxA.predicate($value) || impl_predicate_for_tuple!( @ $self $value $($IdxN),* )
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

/// ASCII newline `\n`.
#[inline(always)]
pub const fn is_newline(ch: &char) -> bool {
    matches!(ch, '\n')
}
/// ASCII whitespace.
///
/// Note that this is different from [`char::is_ascii_whitespace`].
/// This includes U+000B VERTICAL TAB.
#[inline(always)]
pub const fn is_whitespace(ch: &char) -> bool {
    matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b' | '\n')
}
/// [ASCII whitespace](is_whitespace) with No Newline.
#[inline(always)]
pub const fn is_whitespace_nn(ch: &char) -> bool {
    matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b')
}

/// Any ASCII character.
#[inline(always)]
pub const fn is_ascii(ch: &char) -> bool {
    ch.is_ascii()
}
/// ASCII uppercase `[A-Z]`.
#[inline(always)]
pub const fn is_uppercase(ch: &char) -> bool {
    ch.is_ascii_uppercase()
}
/// ASCII lowercase `[a-z]`.
#[inline(always)]
pub const fn is_lowercase(ch: &char) -> bool {
    ch.is_ascii_lowercase()
}
/// ASCII alphabetic `[A-Za-z]`.
#[inline(always)]
pub const fn is_alphabetic(ch: &char) -> bool {
    ch.is_ascii_alphabetic()
}
/// ASCII alphanumeric `[0-9A-Za-z]`.
#[inline(always)]
pub const fn is_alphanumeric(ch: &char) -> bool {
    ch.is_ascii_alphanumeric()
}

/// ASCII decimal digit `[0-9]`.
#[inline(always)]
pub const fn is_digit(ch: &char) -> bool {
    ch.is_ascii_digit()
}
/// ASCII hexadecimal digit `[0-9A-Fa-f]`.
#[inline(always)]
pub const fn is_hexdigit(ch: &char) -> bool {
    ch.is_ascii_hexdigit()
}
/// ASCII octal digit `[0-7]`.
#[inline(always)]
pub const fn is_octdigit(ch: &char) -> bool {
    matches!(ch, '0'..='7')
}
/// ASCII binary digit `[0-1]`.
#[inline(always)]
pub const fn is_bindigit(ch: &char) -> bool {
    matches!(ch, '0' | '1')
}

/// Unicode XID_Start character.
#[inline(always)]
pub fn is_xid_start(ch: &char) -> bool {
    unicode_ident::is_xid_start(*ch)
}

/// Unicode XID_Continue character.
#[inline(always)]
pub fn is_xid_continue(ch: &char) -> bool {
    unicode_ident::is_xid_continue(*ch)
}
