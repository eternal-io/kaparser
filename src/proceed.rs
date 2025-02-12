#![allow(clippy::let_unit_value)]
use super::{common::*, predicate::*};
use core::num::NonZeroUsize;

#[doc(inline)]
pub use crate::token_set;

/// TODO:
/// - `Err(Some(_))` 应该返回还需要多少字节，而不是至少需要多少字节。
/// - `Err(None)` 表明不知道需要多少长度，但暗示了字符串至少应该非空。
///   此时，应该至少读取一个长度的字节。
///   如果遇到了 EOF 之后的下一次尝试仍然如此返回，说明很可能是 TILL/UNTIL 组合器……需要额外提供 EOF 标志吗？
pub type ProceedResult = Result<Transfer, Option<NonZeroUsize>>;

/// Match a set of slices of items (`&str`, `&[u8]`, `&[T]`, [custom](crate::token_set)).
pub trait Proceed<'i, U: ?Sized + Slice> {
    type Captured;
    type Internal: Clone;

    /// `[T; N]` doesn't implement `Default`, so we have to initialize it manually.
    fn init() -> Self::Internal;

    /// 一旦返回了 `Ok(_)`，则不再具有可重入性。
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult;

    /// # Safety
    /// # Panics
    ///
    /// 只在 [`proceed`](Proceed::proceed) 返回 `Ok(Accepted(_))` 时才保证一定能够返回正确结果，
    /// 否则，可能是无意义的结果，甚至 panic。
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

//------------------------------------------------------------------------------

impl<'i> Proceed<'i, str> for &str {
    type Captured = &'i str;
    type Internal = ();

    #[inline(always)]
    fn init() -> Self::Internal {
        ()
    }

    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        (slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len())))
            .ok_or(Some((self.len() - slice.len()).try_into().unwrap()))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        &slice[..self.len()]
    }
}

impl<'i, T: 'i + PartialEq> Proceed<'i, [T]> for &[T] {
    type Captured = &'i [T];
    type Internal = ();

    #[inline(always)]
    fn init() -> Self::Internal {
        ()
    }

    #[inline(always)]
    fn proceed(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        (slice.len() >= self.len())
            .then(|| Transfer::perhaps(slice.starts_with(self).then_some(self.len())))
            .ok_or(Some((self.len() - slice.len()).try_into().unwrap()))
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        &slice[..self.len()]
    }
}

//------------------------------------------------------------------------------

impl<'i, P: Predicate<char>> Proceed<'i, str> for [P; 1] {
    type Captured = char;
    type Internal = ();

    #[inline(always)]
    fn init() -> Self::Internal {
        ()
    }

    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        slice
            .chars()
            .next()
            .map(|ch| Transfer::perhaps(self[0].predicate(&ch).then(|| ch.len_utf8())))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.chars().next().unwrap()
    }
}

impl<'i, T: Clone, P: Predicate<T>> Proceed<'i, [T]> for [P; 1] {
    type Captured = T;
    type Internal = ();

    #[inline(always)]
    fn init() -> Self::Internal {
        ()
    }

    #[inline(always)]
    fn proceed(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let _ = eof;
        let _ = entry;
        slice
            .first()
            .map(|value| Transfer::perhaps(self[0].predicate(value).then_some(1)))
            .ok_or(None)
    }

    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.first().unwrap().clone()
    }
}
