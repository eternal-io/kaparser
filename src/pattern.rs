#![allow(clippy::let_unit_value)]
use crate::{common::*, predicate::*};

#[doc(inline)]
pub use crate::token_set;

/// Match a set of slices of items (`&str`, `&[u8]`, `&[T]`, [custom](crate::token_set)).
pub trait Pattern<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;
    type Internal: 'static + Clone;

    /// `[T; N]` doesn't implement `Default`, so we have to initialize it manually.
    fn init(&self) -> Self::Internal;

    /// 一旦返回了 `Some(_)`，则不再具有可重入性。
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)>;

    /// # Panics
    ///
    /// 只在 [`precede`](Precede::precede) 返回 `Some(Accepted(_))` 时才保证一定能够返回正确结果，
    /// 否则，可能是无意义的结果，甚至 panic。
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U> Pattern<'i, U> for &U
where
    U: 'i + ?Sized + Slice,
{
    type Captured = &'i U;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        (eof || slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len()).ok_or(0)))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.len()).0
    }
}

//------------------------------------------------------------------------------

impl<'i, P> Pattern<'i, str> for [P; 1]
where
    P: Predicate<char>,
{
    type Captured = char;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: &str, _ntry: &mut Self::Internal, _of: bool) -> Option<(Transfer, usize)> {
        slice
            .chars()
            .next()
            .map(|ch| Transfer::perhaps(self[0].predicate(&ch).then(|| ch.len_utf8()).ok_or(0)))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, _ntry: Self::Internal) -> Self::Captured {
        slice.chars().next().unwrap()
    }
}

impl<'i, T, P> Pattern<'i, [T]> for [P; 1]
where
    T: 'i + Copy + PartialEq,
    P: Predicate<T>,
{
    type Captured = T;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: &[T], _ntry: &mut Self::Internal, _of: bool) -> Option<(Transfer, usize)> {
        slice
            .first()
            .map(|value| Transfer::perhaps(self[0].predicate(value).then_some(1).ok_or(0)))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], _ntry: Self::Internal) -> Self::Captured {
        *slice.first().unwrap()
    }
}
