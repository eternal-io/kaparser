use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn want_more(&self, times: usize) -> bool;
}

#[rustfmt::skip]
mod urange_bounds {
    use super::*;
    impl URangeBounds for usize {
        fn contains(&self, times: usize) -> bool { times == *self }
        fn want_more(&self, times: usize) -> bool { times < *self }
    }
    impl URangeBounds for RangeFull {
        fn contains(&self, _t: usize) -> bool { true }
        fn want_more(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for RangeFrom<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for Range<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times < *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times < self.end }
    }
}

//==================================================================================================

/// Trait for predicates a set of characters.
pub trait Predicate {
    fn predicate(&self, ch: char) -> bool;
}

impl<T: 'static + Fn(char) -> bool> Predicate for T {
    fn predicate(&self, ch: char) -> bool {
        self(ch)
    }
}

impl Predicate for char {
    fn predicate(&self, ch: char) -> bool {
        ch == *self
    }
}
impl Predicate for &[char] {
    fn predicate(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}
impl<const N: usize> Predicate for [char; N] {
    fn predicate(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}
impl Predicate for Range<char> {
    fn predicate(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}
impl Predicate for RangeInclusive<char> {
    fn predicate(&self, ch: char) -> bool {
        self.contains(&ch)
    }
}

/// Trait for matches a set of strings.
///
/// 这个特质被标记为 unsafe，你永远也不应该手动实施它，转而使用 [`token_set!`] 宏。
///
/// # Safety
///
/// TODO: 转而使用 [`token_set!`] 宏。
pub unsafe trait Pattern {
    type Discriminant;

    /// Returns the max length of possible sub-patterns.
    fn max_len(&self) -> usize;

    /// Returns the length of the matched sub-pattern in bytes,
    /// and the discriminant of the sub-pattern.
    fn matches(&self, content: &str) -> Option<(usize, Self::Discriminant)>;
}

unsafe impl<'a> Pattern for &'a str {
    type Discriminant = &'a str;

    fn max_len(&self) -> usize {
        self.len()
    }

    fn matches(&self, content: &str) -> Option<(usize, Self::Discriminant)> {
        content.starts_with(self).then_some((self.len(), self))
    }
}

//==================================================================================================

macro_rules! impl_predicate_for_tuple {
    ( $( $Pn:ident ~ $n:tt )* ) => {
        impl<$($Pn: Predicate),*> Predicate for ($($Pn,)*) {
            #[inline(always)]
            fn predicate(&self, _ch: char) -> bool {
                impl_predicate_for_tuple!( @ self _ch $($n),* )
            }
        }
    };

    ( @ $self:ident $ch:ident ) => { false };

    ( @ $self:ident $ch:ident $n:tt ) => {
        $self.$n.predicate($ch)
    };

    ( @ $self:ident $ch:ident $n:tt, $($n_:tt),* ) => {
        $self.$n.predicate($ch) || impl_predicate_for_tuple!( @ $self $ch $($n_),* )
    };
}

macro_rules! impl_predicate_for_tuples {
    ( $( $Pn:ident ~ $n:tt )* ) => {
        impl_predicate_for_tuples!( @ ; $($Pn ~ $n)* );
    };

    ( @ $( $Pn:ident ~ $n:tt )* ; $Pk:ident ~ $k:tt $( $Pm:ident ~ $m:tt )* ) => {
            impl_predicate_for_tuple! ( $($Pn ~ $n)* );
        impl_predicate_for_tuples!( @   $($Pn ~ $n)* $Pk ~ $k ; $($Pm ~ $m)* );
    };

    ( @ $( $Pn:ident ~ $n:tt )* ; ) => {
            impl_predicate_for_tuple! ( $($Pn ~ $n)* );
    };
}

// _=(lambda x: [print(f'P{i} ~ {i}') for i in range(x)])(16)
impl_predicate_for_tuples! {
    P0 ~ 0
    P1 ~ 1
    P2 ~ 2
    P3 ~ 3
    P4 ~ 4
    P5 ~ 5
    P6 ~ 6
    P7 ~ 7
    P8 ~ 8
    P9 ~ 9
    P10 ~ 10
    P11 ~ 11
    P12 ~ 12
    P13 ~ 13
    P14 ~ 14
    P15 ~ 15
}

/// Combine predicates, produce a new predicate that accepts any character except these specified.
#[macro_export]
macro_rules! not {
    ( $($preds:expr),* $(,)? ) => {
        move |ch: char| -> bool {
            not!( @ ch $($preds),* )
        }
    };

    ( @ $ch:ident ) => { true };

    ( @ $ch:ident $pred:expr ) => {
        !$pred.predicate($ch)
    };

    ( @ $ch:ident $pred:expr, $($preds:expr),* ) => {
        !$pred.predicate($ch) && not!( @ $ch $($preds),* )
    };
}

/// Generate structures, implement [`Pattern`] for a set of tokens conveniently.
#[macro_export]
macro_rules! token_set {
    ( $(
        $(#[$attr:meta])*
        $name:ident { $(
            $(#[$bttr:meta])*
            $key:ident = $word:literal
        ),* $(,)? }
    )* ) => { $( $crate::paste! {
      $(#[$attr])*
        #[doc = "\n\n*(generated token discriminant)*"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) enum [<$name Token>] { $(
          $(#[$bttr])*
            #[doc = "\n\nAssociates `` " $word " ``"]
            $key,
        )* }

        impl [<$name Token>] {
            /// Returns the associated text.
            #[allow(dead_code, unreachable_patterns)]
            pub fn text(&self) -> &'static str {
                match self {
                    $( Self::$key => token_set!( @validate $word ), )*
                    _ => unreachable!(),
                }
            }
        }

        #[doc = "Generated tokens pattern with [`" [<$name Token>] "`] discriminant."
                "\n\nZST type by [`token_set!`] macro, only for passing as argument."]
        pub(crate) struct [<$name Tokens>];

        unsafe impl Pattern for [<$name Tokens>] {
            type Discriminant = [<$name Token>];

            fn max_len(&self) -> usize {
                const { token_set!( @max $($word.len(),)* 0 ) }
            }

            fn matches(&self, _content: &str) -> Option<(usize, Self::Discriminant)> {
            $(
                if _content.starts_with($word) {
                    return const { Some(($word.len(), Self::Discriminant::$key)) }
                }
            )*
                None
            }
        }
    } )* };

    ( @max $expr:expr ) => { $expr };

    ( @max $expr:expr, $( $exprs:expr ),+ ) => {{
        let a = $expr;
        let b = token_set!( @max $($exprs),+ );

        if a > b { a } else { b }
    }};

    ( @validate $word:literal ) => {
        const {
            let word = $word;
            assert!(
                !word.is_empty() && word.len() <= 8192,
                "the associated text must be non-empty string, and no more than 8192 bytes"
            );
            word
        }
    };
}

//==================================================================================================

/// Any character.
#[inline(always)]
pub const fn any(ch: char) -> bool {
    let _ = ch;
    true
}

/// ASCII newline `\n`.
#[inline(always)]
pub const fn is_newline(ch: char) -> bool {
    ch == '\n'
}

/// ASCII whitespace.
///
/// Note that this is different from [`char::is_ascii_whitespace`].
/// This includes U+000B VERTICAL TAB.
#[inline(always)]
pub const fn is_whitespace(ch: char) -> bool {
    matches!(ch, '\n' | '\t' | '\r' | '\x0b' | '\x0c' | '\x20')
}

/// [ASCII whitespace](is_whitespace) with No Newline.
#[inline(always)]
pub const fn is_whitespace_nn(ch: char) -> bool {
    matches!(ch, '\n' | '\t' | '\r' | '\x0b' | '\x0c' | '\x20')
}

/// Any ASCII character.
#[inline(always)]
pub const fn is_ascii(ch: char) -> bool {
    ch.is_ascii()
}
/// ASCII alphabetic `[A-Za-z]`.
#[inline(always)]
pub const fn is_alphabetic(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}
/// ASCII alphanumeric `[0-9A-Za-z]`.
#[inline(always)]
pub const fn is_alphanumeric(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

/// ASCII decimal digit `[0-9]`.
#[inline(always)]
pub const fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}
/// ASCII hexadecimal digit `[0-9A-Fa-f]`.
#[inline(always)]
pub const fn is_hexdigit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}
/// ASCII octal digit `[0-7]`.
#[inline(always)]
pub const fn is_octdigit(ch: char) -> bool {
    matches!(ch, '0'..='7')
}
/// ASCII binary digit `[0-1]`.
#[inline(always)]
pub const fn is_bindigit(ch: char) -> bool {
    matches!(ch, '0' | '1')
}

/// Unicode XID_Start character.
#[inline(always)]
pub fn is_xid_start(ch: char) -> bool {
    unicode_ident::is_xid_start(ch)
}

/// Unicode XID_Continue character.
#[inline(always)]
pub fn is_xid_continue(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch)
}
