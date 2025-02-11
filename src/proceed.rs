#![allow(clippy::let_unit_value)]
use super::{common::*, *};
use core::num::NonZeroUsize;

/// TODO:
/// - `Err(Some(_))` 应该返回还需要多少字节，而不是至少需要多少字节。
/// - `Err(None)` 表明不知道需要多少长度，但暗示了字符串至少应该非空。
///   此时，应该至少读取一个长度的字节。
///   如果遇到了 EOF 之后的下一次尝试仍然如此返回，说明很可能是 TILL/UNTIL 组合器……需要额外提供 EOF 标志吗？
pub type ProceedResult = Result<Transfer, Option<NonZeroUsize>>;

/// Match a set of slices of items (`&str`, `&[u8]`, `&[T]`, [custom](crate::token_set)).
pub trait Proceed<'i, U: ?Sized + Slice> {
    type Capture;
    type State: Default;

    fn proceed(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult;

    /// # Safety
    /// # Panics
    ///
    /// 只在 [`proceed`](Proceed::proceed) 返回 `Ok(Accepted(_))` 时才保证一定能够返回正确结果，
    /// 否则，可能是无意义的结果，甚至 panic。
    fn extract(&self, slice: &'i U, entry: Self::State) -> Self::Capture;
}

//------------------------------------------------------------------------------

impl<'i> Proceed<'i, str> for &str {
    type Capture = &'i str;
    type State = ();

    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::State, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        (slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len())))
            .ok_or(Some((self.len() - slice.len()).try_into().unwrap()))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::State) -> Self::Capture {
        let _ = entry;
        &slice[..self.len()]
    }
}

impl<'i, T: 'i + PartialEq> Proceed<'i, [T]> for &[T] {
    type Capture = &'i [T];
    type State = ();

    #[inline(always)]
    fn proceed(&self, slice: &'i [T], entry: &mut Self::State, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        (slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len())))
            .ok_or(Some((self.len() - slice.len()).try_into().unwrap()))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::State) -> Self::Capture {
        let _ = entry;
        &slice[..self.len()]
    }
}

//------------------------------------------------------------------------------

impl<'i, P: Predicate<char>> Proceed<'i, str> for P {
    type Capture = char;
    type State = ();

    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::State, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        slice
            .chars()
            .next()
            .map(|ch| Transfer::perhaps(self.predicate(&ch).then(|| ch.len_utf8())))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::State) -> Self::Capture {
        let _ = entry;
        slice.chars().next().unwrap()
    }
}

impl<'i, T: Clone, P: Predicate<T>> Proceed<'i, [T]> for [P; 1] {
    type Capture = T;
    type State = ();

    #[inline(always)]
    fn proceed(&self, slice: &'i [T], entry: &mut Self::State, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        slice
            .first()
            .map(|value| Transfer::perhaps(self[0].predicate(value).then_some(1)))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::State) -> Self::Capture {
        let _ = entry;
        slice.first().unwrap().clone()
    }
}

//------------------------------------------------------------------------------

#[macro_export]
macro_rules! token_set {
    () => {};
}

// /// Generate structures, implement [`Proceed`] for a set of tokens conveniently.
// #[macro_export]
// macro_rules! token_set {
//     ( $(
//         $(#[$attr:meta])*
//         $name:ident { $(
//             $(#[$bttr:meta])*
//             $key:ident = $word:literal
//         ),* $(,)? }
//     )* ) => { $( $crate::common::paste! {
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
//         impl Proceed for [<$name Tokens>] {
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
