use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn want_more(&self, times: usize) -> bool;
}

#[rustfmt::skip]
impl URangeBounds for usize {
    fn contains(&self, times: usize) -> bool { times == *self }
    fn want_more(&self, times: usize) -> bool { times < *self }
}
#[rustfmt::skip]
impl URangeBounds for RangeFull {
    fn contains(&self, _t: usize) -> bool { true }
    fn want_more(&self, _t: usize) -> bool { true }
}
#[rustfmt::skip]
impl URangeBounds for RangeFrom<usize> {
    fn contains(&self, times: usize) -> bool { self.contains(&times) }
    fn want_more(&self, _t: usize) -> bool { true }
}
#[rustfmt::skip]
impl URangeBounds for Range<usize> {
    fn contains(&self, times: usize) -> bool { self.contains(&times) }
    fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
}
#[rustfmt::skip]
impl URangeBounds for RangeTo<usize> {
    fn contains(&self, times: usize) -> bool { self.contains(&times) }
    fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
}
#[rustfmt::skip]
impl URangeBounds for RangeInclusive<usize> {
    fn contains(&self, times: usize) -> bool { self.contains(&times) }
    fn want_more(&self, times: usize) -> bool { times < *self.end() }
}
#[rustfmt::skip]
impl URangeBounds for RangeToInclusive<usize> {
    fn contains(&self, times: usize) -> bool { self.contains(&times) }
    fn want_more(&self, times: usize) -> bool { times < self.end }
}

//------------------------------------------------------------------------------

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
pub trait Pattern {
    type Discriminant;

    /// Returns the max length of possible sub-patterns.
    fn max_len(&self) -> usize;

    /// Returns the length of the matched sub-pattern in bytes,
    /// and the discriminant of the sub-pattern.
    fn matches(&self, content: &str) -> Option<(usize, Self::Discriminant)>;

    fn full_match(&self, content: &str) -> Option<Self::Discriminant> {
        self.matches(content)
            .and_then(|(len, discr)| (len == 0).then_some(discr))
    }
}

impl<'a> Pattern for &'a str {
    type Discriminant = &'a str;

    fn max_len(&self) -> usize {
        self.len()
    }

    fn matches(&self, content: &str) -> Option<(usize, Self::Discriminant)> {
        content.starts_with(self).then_some((self.len(), self))
    }
}

//==================================================================================================

/// Combine predicates, produce a new predicate that accepts only these specified characters.
#[macro_export]
macro_rules! all {
    ( $($preds:expr),+ $(,)? ) => {
        move |ch: char| all!( @ ch $($preds),+ )
    };

    ( @ $ch:ident $pred:expr, $($preds:expr),* ) => {
        $pred.predicate($ch) || all!( @ $ch $($preds),* )
    };

    ( @ $ch:ident $pred:expr ) => {
        $pred.predicate($ch)
    };
}

/// Combine predicates, produce a new predicate that accepts any character except these specified.
#[macro_export]
macro_rules! not {
    ( $($preds:expr),+ $(,)? ) => {
        move |ch: char| not!( @ ch $($preds),+ )
    };

    ( @ $ch:ident $pred:expr, $($preds:expr),* ) => {
        !$pred.predicate($ch) && not!( @ $ch $($preds),* )
    };

    ( @ $ch:ident $pred:expr ) => {
        !$pred.predicate($ch)
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

        impl Pattern for [<$name Tokens>] {
            type Discriminant = [<$name Token>];

            fn max_len(&self) -> usize {
                token_set!( @max $($word.len(),)* 0 )
            }

            #[allow(unused_variables)]
            fn matches(&self, content: &str) -> Option<(usize, Self::Discriminant)> {
            $(
                if content.starts_with($word) {
                    return Some(($word.len(), Self::Discriminant::$key))
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

    ( @validate "" ) => { compile_error!("the associated text must be non-empty string") };

    ( @validate $word:literal ) => { $word };
}

//==================================================================================================

/// Any character.
pub const fn any(ch: char) -> bool {
    let _ = ch;
    true
}

/// ASCII newline `\n`.
pub const fn is_newline(ch: char) -> bool {
    ch == '\n'
}

/// ASCII whitespace.
///
/// Note that this is different from [`char::is_ascii_whitespace`].
/// This includes U+000B VERTICAL TAB.
pub const fn is_whitespace(ch: char) -> bool {
    matches!(ch, '\n' | '\t' | '\r' | '\x0b' | '\x0c' | '\x20')
}

/// Any ASCII character.
pub const fn is_ascii(ch: char) -> bool {
    ch.is_ascii()
}
/// ASCII alphabetic `[A-Za-z]`.
pub const fn is_alphabetic(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}
/// ASCII alphanumeric `[0-9A-Za-z]`.
pub const fn is_alphanumeric(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

/// ASCII decimal digit `[0-9]`.
pub const fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}
/// ASCII hexadecimal digit `[0-9A-Fa-f]`.
pub const fn is_hexdigit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}
/// ASCII octal digit `[0-7]`.
pub const fn is_octdigit(ch: char) -> bool {
    matches!(ch, '0'..='7')
}
/// ASCII binary digit `[0-1]`.
pub const fn is_bindigit(ch: char) -> bool {
    matches!(ch, '0' | '1')
}

/// Unicode XID_START character.
pub fn is_xid_start(ch: char) -> bool {
    unicode_ident::is_xid_start(ch)
}

/// Unicode XID_CONTINUE character.
pub fn is_xid_continue(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch)
}
