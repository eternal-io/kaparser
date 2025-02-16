#![allow(clippy::let_unit_value)]
use crate::{common::*, predicate::*};
use core::num::NonZeroUsize;

#[doc(inline)]
pub use crate::token_set;

pub type PrecedeResult = Result<(Transfer, usize), Option<NonZeroUsize>>;

/// Match a set of slices of items (`&str`, `&[u8]`, `&[T]`, [custom](crate::token_set)).
pub trait Pattern<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;
    type Internal: 'static + Clone;

    /// `[T; N]` doesn't implement `Default`, so we have to initialize it manually.
    fn init(&self) -> Self::Internal;

    /// 一旦返回了 `Ok(_)`，则不再具有可重入性。
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult;

    /// # Panics
    ///
    /// 只在 [`precede`](Precede::precede) 返回 `Ok(Accepted(_))` 时才保证一定能够返回正确结果，
    /// 否则，可能是无意义的结果，甚至 panic。
    ///
    /// TODO: 修改函数签名和调用约定，允许重复调用
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
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let _ = eof;
        let _ = entry;
        (slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len()).ok_or(0)))
            .ok_or_else(|| Some((self.len() - slice.len()).try_into().unwrap()))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let _ = entry;
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
    fn precede(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let _ = eof;
        let _ = entry;
        slice
            .chars()
            .next()
            .map(|ch| Transfer::perhaps(self[0].predicate(&ch).then(|| ch.len_utf8()).ok_or(0)))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.chars().next().unwrap()
    }
}

impl<'i, T, P> Pattern<'i, [T]> for [P; 1]
where
    T: 'i + PartialEq,
    P: Predicate<T>,
{
    type Captured = &'i T;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let _ = eof;
        let _ = entry;
        slice
            .first()
            .map(|value| Transfer::perhaps(self[0].predicate(value).then_some(1).ok_or(0)))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.first().unwrap()
    }
}
