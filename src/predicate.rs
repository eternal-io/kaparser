use crate::{common::*, error::*, extra::*, input::*, pattern::*, primitive};
use core::{
    any::type_name,
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Range, RangeInclusive},
};

pub struct ANY;

pub struct Just<Token: PartialEq + Debug>(pub Token);

pub struct Except<Pred>(Pred);

pub fn except<Token, Pred>(pred: Pred) -> impl Predicate<Token>
where
    Pred: Predicate<Token>,
{
    Except(pred)
}

//------------------------------------------------------------------------------

impl<Token> Describe for &dyn Predicate<Token> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "single token matches ")?;
        self.describe(f)
    }
}

pub trait Predicate<Token> {
    fn predicate(&self, item: &Token) -> bool;

    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    #[doc(hidden)]
    fn report<E: Error>(&self, span: Range<usize>) -> E
    where
        Self: Sized,
    {
        E::new(
            span,
            ErrorKind::Expected(coerce_dyn! { self => Predicate<Token> => Describe }),
        )
    }

    fn take<'src, I, Ext, R>(self, range: R) -> impl Pattern<'src, I, Ext>
    where
        Self: Sized + Predicate<I::Token>,
        I: InputSlice<'src>,
        Ext: Extra<'src, I>,
        R: URangeBounds,
    {
        primitive::Take {
            pred: self,
            range,
            phantom: PhantomData,
        }
    }

    fn take0more<'src, I, Ext>(self) -> impl Pattern<'src, I, Ext>
    where
        Self: Sized + Predicate<I::Token>,
        I: InputSlice<'src>,
        Ext: Extra<'src, I>,
    {
        primitive::Take {
            pred: self,
            range: ..,
            phantom: PhantomData,
        }
    }

    fn take1more<'src, I, Ext>(self) -> impl Pattern<'src, I, Ext>
    where
        Self: Sized + Predicate<I::Token>,
        I: InputSlice<'src>,
        Ext: Extra<'src, I>,
    {
        primitive::Take {
            pred: self,
            range: OneOrMore,
            phantom: PhantomData,
        }
    }
}

impl<Token> Predicate<Token> for ANY {
    fn predicate(&self, item: &Token) -> bool {
        #![allow(unused_variables)]
        true
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ANY")
    }
}

impl<Token: PartialEq + Debug> Predicate<Token> for Just<Token> {
    fn predicate(&self, item: &Token) -> bool {
        self.0.eq(item)
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<Token, Pred: Predicate<Token>> Predicate<Token> for Except<Pred> {
    fn predicate(&self, item: &Token) -> bool {
        !self.0.predicate(item)
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "except ")?;
        self.0.describe(f)
    }
}

impl<Token, F: Fn(&Token) -> bool> Predicate<Token> for F {
    fn predicate(&self, item: &Token) -> bool {
        self(item)
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "guard {}", type_name::<F>())
    }
}

impl<Token: PartialOrd + Debug> Predicate<Token> for Range<Token> {
    fn predicate(&self, item: &Token) -> bool {
        self.contains(item)
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl<Token: PartialOrd + Debug> Predicate<Token> for RangeInclusive<Token> {
    fn predicate(&self, item: &Token) -> bool {
        self.contains(item)
    }
    fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..={:?}", self.start(), self.end())
    }
}

macro_rules! impl_predicate_for_primitives {
    ( $($ty:ty),+ $(,)? ) => { $(
        impl Predicate<$ty> for $ty {
            fn predicate(&self, item: &$ty) -> bool {
                self.eq(item)
            }
            fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    )+ };
}

impl_predicate_for_primitives! {
    bool, char,
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
             f32, f64,
}

macro_rules! impl_predicate_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => {
        impl<Token, $($GenN),+> Predicate<Token> for ($($GenN,)+)
        where
          $($GenN: Predicate<Token>,)+
        {
            fn predicate(&self, item: &Token) -> bool {
                impl_predicate_for_tuple!( @pred self item $($IdxN),+ )
            }
            fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("{ ")?;
                impl_predicate_for_tuple!( @desc self f $($IdxN),+ )?;
                f.write_str(" }")
            }
        }
    };

    ( @pred $self:ident $item:ident $IdxA:tt ) => {
        $self.$IdxA.predicate($item)
    };

    ( @pred $self:ident $item:ident $IdxA:tt, $($IdxN:tt),* ) => {
        $self.$IdxA.predicate($item) || impl_predicate_for_tuple!( @pred $self $item $($IdxN),* )
    };

    ( @desc $self:ident $fmtr:ident $IdxA:tt ) => {
        $self.$IdxA.describe($fmtr)
    };

    ( @desc $self:ident $fmtr:ident $IdxA:tt, $($IdxN:tt),* ) => {{
        $self.$IdxA.describe($fmtr)?;
        $fmtr.write_str(" | ")?;
        impl_predicate_for_tuple!( @desc $self $fmtr $($IdxN),* )
    }};
}

__generate_codes! { impl_predicate_for_tuple ( P ) }

//------------------------------------------------------------------------------

macro_rules! gen_ascii_predicates {
    ( $(
      $(#[$attr:meta])*
        $desc:literal $func:ident($ch:ident) => $expr:expr
    ),* $(,)? ) => { paste::paste! { $(
        #[doc = "ASCII " $desc " character.\n\n"]
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

// TODO: provide structs instead, because `Fn`s are not describable.

gen_ascii_predicates! {
    /// U+0000 NUL ..= U+001F UNIT SEPARATOR, or U+007F DELETE.
    r"control"              is_ascii_control(ch)        => ch.is_ascii_control(),

    /// U+0021 '!' ..= U+007E '~'.
    r"printable"            is_ascii_graphic(ch)        => ch.is_ascii_graphic(),

    /// - U+0021 ..= U+002F `! " # $ % & ' ( ) * + , - . /`, or
    /// - U+003A ..= U+0040 `: ; < = > ? @`, or
    /// - U+005B ..= U+0060 ``[ \ ] ^ _ ` ``, or
    /// - U+007B ..= U+007E `{ | } ~`
    r"punctuation"          is_ascii_punctuation(ch)    => ch.is_ascii_punctuation(),

    /// U+0020 SPACE, U+0009 HORIZONTAL TAB, U+000A LINE FEED,
    /// U+000B VERTICAL TAB, U+000C FORM FEED, or U+000D CARRIAGE RETURN.
    r"whitespace"           is_ascii_whitespace(ch)     => matches!(ch, '\x20' | '\t' | '\r' | '\x0c' | '\x0b' | '\n'),

    r"any"                  is_ascii(ch)                => ch.is_ascii(),
    r"uppercase"            is_ascii_uppercase(ch)      => ch.is_ascii_uppercase(),
    r"lowercase"            is_ascii_lowercase(ch)      => ch.is_ascii_lowercase(),
    r"alphabetic"           is_ascii_alphabetic(ch)     => ch.is_ascii_alphabetic(),
    r"alphanumeric"         is_ascii_alphanumeric(ch)   => ch.is_ascii_alphanumeric(),

    r"decimal digit"        is_ascii_digit(ch)          => ch.is_ascii_digit(),
    r"hexadecimal digit"    is_ascii_hexdigit(ch)       => ch.is_ascii_hexdigit(),
    r"octal digit"          is_ascii_octdigit(ch)       => matches!(ch, '0'..='7'),
    r"binary digit"         is_ascii_bindigit(ch)       => matches!(ch, '0' | '1'),
}

gen_unicode_predicates! {
    "with `XID_Start` property"         is_xid_start(ch)    => unicode_ident::is_xid_start(*ch),
    "with `XID_Continue` property"      is_xid_continue(ch) => unicode_ident::is_xid_continue(*ch),

    "with `White_Space` property"       is_unicode_whitespace(ch)   => ch.is_whitespace(),
    "with `Lowercase` property"         is_unicode_lowercase(ch)    => ch.is_lowercase(),
    "with `Uppercase` property"         is_unicode_uppercase(ch)    => ch.is_uppercase(),
    "with `Alphabetic` property"        is_unicode_alphabetic(ch)   => ch.is_alphabetic(),
    "with `Alphabetic` property or in general numbers categories"
                                        is_unicode_alphanumeric(ch) => ch.is_alphanumeric(),
    "in general numbers categories"     is_unicode_numeric(ch)      => ch.is_numeric(),
    "in general control codes category" is_unicode_control(ch)      => ch.is_control(),
}
