#![allow(clippy::let_unit_value)]
use crate::{common::*, predicate::*};

#[doc(inline)]
pub use crate::token_set;

pub trait Pattern2<U>
where
    U: Slice2,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init2(&self) -> Self::Internal;
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)>;
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured;
}

impl<U> Pattern2<U> for &U
where
    U: Slice2,
{
    type Captured = ();
    type Internal = ();

    fn init2(&self) -> Self::Internal {
        todo!()
    }

    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        todo!()
    }

    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        todo!()
    }
}

//==================================================================================================

/// Match a set of slices of items (`&str`, `&[u8]`, `&[T]`, [custom](crate::token_set)).
pub trait Pattern<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;
    type Internal: 'static + Clone;

    /// `[T; N]` doesn't implement `Default`, so we have to initialize it manually.
    fn init(&self) -> Self::Internal;

    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)>;

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
