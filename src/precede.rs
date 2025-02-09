use crate::{common::*, Predicate};

pub type PrecedeResult<Alternate, Checkpoint> = Result<Option<(usize, Alternate)>, (usize, Checkpoint)>;

/// Trait for matches a set of slices of items.
pub trait Precede<'i, U: ?Sized> {
    type Alternate;

    type Checkpoint: Default;

    fn precede(&self, slice: &'i U, resume: Self::Checkpoint) -> PrecedeResult<Self::Alternate, Self::Checkpoint>;
}

/* IDEA: I thought this is one of the most important blanket implement. */
impl<'i, P: Predicate<char>> Precede<'i, str> for P {
    type Alternate = char;
    type Checkpoint = ();
    #[inline(always)]
    fn precede(&self, slice: &'i str, resume: Self::Checkpoint) -> PrecedeResult<Self::Alternate, Self::Checkpoint> {
        let _ = resume;
        slice
            .chars()
            .next()
            .map(|ch| self.predicate(&ch).then_some((ch.len_utf8(), ch)))
            .ok_or((4, ()))
    }
}

impl<'i, T: 'i, P: Predicate<T>> Precede<'i, [T]> for P {
    type Alternate = &'i T;
    type Checkpoint = ();
    #[inline(always)]
    fn precede(&self, slice: &'i [T], resume: Self::Checkpoint) -> PrecedeResult<Self::Alternate, Self::Checkpoint> {
        let _ = resume;
        slice
            .first()
            .map(|value| self.predicate(value).then_some((1, value)))
            .ok_or((1, ()))
    }
}

macro_rules! impl_precede_for_primitives {
    ( $($ty:ty),+$(,)? ) => { $(
        impl<'i> Precede<'i, $ty> for $ty {
            type Alternate = &'i $ty;
            type Checkpoint = ();
            #[inline(always)]
            fn precede(&self, slice: &'i $ty, resume: Self::Checkpoint) -> PrecedeResult<Self::Alternate, Self::Checkpoint> {
                let _ = resume;
                (slice.len() >= self.len())
                    .then(|| slice.starts_with(self).then_some((self.len(), &slice[..self.len()])))
                    .ok_or((self.len(), ()))
            }
        }
    )+ };
}

impl_precede_for_primitives! {
    [bool], [char], str,
    [i8], [i16], [i32], [i64], [isize],
    [u8], [u16], [u32], [u64], [usize],
                 [f32], [f64],
}

// macro_rules! impl_precede_for_tuple {
//     ( $Lk:tt, $( $Pn:ident ~ $n:tt )* ) => { $crate::paste! {
//         impl<'i, U: ?Sized, $($Pn: Precede<'i, U>),*> Precede<'i, U> for ($($Pn,)*) {
//             type Alternate = [<Alternate $Lk s>]<$($Pn::Alternate),*>;
//             type Checkpoint = Option<[<Checkpoint $Lk s>]>;
//             #[inline(always)] #[allow(unused_variables)]
//             fn precede(&self, slice: &'i U, resume: Self::Checkpoint) -> PrecedeResult<Self::Alternate, Self::Checkpoint> {
//                 impl_precede_for_tuple!( @head self value $($n),* )
//             }
//         }
//     } };
//
//     ( @head $self:ident $value:ident ) => { Ok(None) };
//
//     ( @head $self:ident $value:ident $n:tt ) => {
//         todo!()
//         // $self.$n.predicate($value)
//     };
//
//     ( @head $self:ident $value:ident $n:tt, $($n_:tt),* ) => {
//         todo!()
//         // $self.$n.predicate($value) || impl_precede_for_tuple!( @ $self $value $($n_),* )
//     };
//
//     ( @body $self:ident $value:ident $n:tt ) => {
//         todo!()
//     };
// }

// macro_rules! impl_precede_for_tuples {
//     (      $Lk:literal ~ $Pk:ident ~ $k:tt
//         $( $Ln:literal ~ $Pn:ident ~ $n:tt )*
//     ) => {
//         impl_precede_for_tuples!( @ $Lk ~ $Pk ~ $k ; $($Ln ~ $Pn ~ $n)* );
//     };
//
//     ( @ $( $Ln:literal ~ $Pn:ident ~ $n:tt )+ ;
//            $Lk:literal ~ $Pk:ident ~ $k:tt
//         $( $Lm:literal ~ $Pm:ident ~ $m:tt )*
//     ) => {
//         impl_precede_for_tuple!( $Lk, $($Pn ~ $n)+ );
//         impl_precede_for_tuples! { @
//             $($Ln ~ $Pn ~ $n)+
//               $Lk ~ $Pk ~ $k ;
//             $($Lm ~ $Pm ~ $m)*
//         }
//     };
//
//     ( @ $( $Ln:literal ~ $Pn:ident ~ $n:tt )+ ; ) => {};
// }

// impl_precede_for_tuples! {
//     0  ~ P1  ~ 1
//     1  ~ P2  ~ 2
//     2  ~ P3  ~ 3
//     3  ~ P4  ~ 4
//     4  ~ P5  ~ 5
//     5  ~ P6  ~ 6
//     6  ~ P7  ~ 7
//     7  ~ P8  ~ 8
//     8  ~ P9  ~ 9
//     9  ~ P10 ~ 10
//     10 ~ P11 ~ 11
//     11 ~ P12 ~ 12
//     12 ~ P13 ~ 13
//     13 ~ P14 ~ 14
//     14 ~ P15 ~ 15
//     15 ~ P16 ~ 16
//     16 ~ P17 ~ 17
// }

// /// Generate structures, implement [`Precede`] for a set of tokens conveniently.
// #[macro_export]
// macro_rules! token_set {
//     ( $(
//         $(#[$attr:meta])*
//         $name:ident { $(
//             $(#[$bttr:meta])*
//             $key:ident = $word:literal
//         ),* $(,)? }
//     )* ) => { $( $crate::paste! {
//       $(#[$attr])*
//         #[doc = "\n\n*(generated token discriminant)*"]
//         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//         pub(crate) enum [<$name Token>] { $(
//           $(#[$bttr])*
//             #[doc = "\n\nAssociates `` " $word " ``"]
//             $key,
//         )* }
//
//         impl [<$name Token>] {
//             /// Returns the associated text.
//             #[allow(dead_code, unreachable_patterns)]
//             pub fn text(&self) -> &'static str {
//                 match self {
//                     $( Self::$key => token_set!( @validate $word ), )*
//                     _ => unreachable!(),
//                 }
//             }
//         }
//
//         #[doc = "Generated tokens pattern with [`" [<$name Token>] "`] discriminant."
//                 "\n\nZST type by [`token_set!`] macro, only for passing as argument."]
//         pub(crate) struct [<$name Tokens>];
//
//         impl Precede for [<$name Tokens>] {
//             type Discriminant = [<$name Token>];
//
//             fn max_len(&self) -> usize {
//                 const { token_set!( @max $($word.len(),)* 0 ) }
//             }
//
//             fn matches(&self, _content: &str) -> Option<(usize, Self::Discriminant)> {
//             $(
//                 if _content.starts_with($word) {
//                     return const { Some(($word.len(), Self::Discriminant::$key)) }
//                 }
//             )*
//                 None
//             }
//         }
//     } )* };
//
//     ( @max $expr:expr ) => { $expr };
//
//     ( @max $expr:expr, $( $exprs:expr ),+ ) => {{
//         let a = $expr;
//         let b = token_set!( @max $($exprs),+ );
//
//         if a > b { a } else { b }
//     }};
//
//     ( @validate $word:literal ) => {
//         const {
//             let word = $word;
//             assert!(
//                 !word.is_empty() && word.len() <= 8192,
//                 "the associated text must be non-empty string, and no more than 8192 bytes"
//             );
//             word
//         }
//     };
// }
