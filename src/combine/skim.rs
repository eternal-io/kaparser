use super::*;
use core::ops::{RangeTo, RangeToInclusive};

#[inline(always)]
pub const fn till<'i, T, P>(end: P) -> RangeTo<P>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
{
    RangeTo { end }
}

#[inline(always)]
pub const fn until<'i, U, P>(end: P) -> RangeToInclusive<P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    RangeToInclusive { end }
}

//------------------------------------------------------------------------------

impl<'i, P> Pattern<'i, str> for RangeTo<P>
where
    P: Predicate<char>,
{
    type Captured = (&'i str, Option<char>);
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        match slice
            .split_at(*entry)
            .1
            .char_indices()
            .find(|(_, ch)| self.end.predicate(ch))
        {
            Some((off, _)) => {
                *entry += off;
                Ok((Transfer::Accepted, *entry))
            }
            None => {
                *entry = slice.len();
                eof.then_some((Transfer::Accepted, *entry)).ok_or(None)
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.chars().next())
    }
}

impl<'i, T, P> Pattern<'i, [T]> for RangeTo<P>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
{
    type Captured = (&'i [T], Option<&'i T>);
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        match slice
            .split_at(*entry)
            .1
            .iter()
            .enumerate()
            .find(|(_, value)| self.end.predicate(value))
        {
            Some((off, _)) => {
                *entry += off;
                Ok((Transfer::Accepted, *entry))
            }
            None => {
                *entry = slice.len();
                eof.then_some((Transfer::Accepted, *entry)).ok_or(None)
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.iter().next())
    }
}

//------------------------------------------------------------------------------

impl<'i, P> Pattern<'i, str> for RangeToInclusive<P>
where
    P: Pattern<'i, str>,
{
    type Captured = (&'i str, P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.end.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (off, state) = entry;
        for ch in slice.split_at(*off).1.chars() {
            let (t, len) = self.end.precede(slice.split_at(*off).1, state, eof)?;
            match t {
                Transfer::Rejected => (),
                t => return Ok((t, len)),
            }
            *off += ch.len_utf8();
        }
        eof.then_some((Transfer::Rejected, *off)).ok_or(None)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        let (off, state) = entry;
        let (left, right) = slice.split_at(off);
        (left, self.end.extract(right, state))
    }
}

impl<'i, T, P> Pattern<'i, [T]> for RangeToInclusive<P>
where
    T: 'i + PartialEq,
    P: Pattern<'i, [T]>,
{
    type Captured = (&'i [T], P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.end.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (off, state) = entry;
        while *off < slice.len() {
            let (t, len) = self.end.precede(slice.split_at(*off).1, state, eof)?;
            match t {
                Transfer::Rejected => (),
                t => return Ok((t, len)),
            }
            *off += 1;
        }
        eof.then_some((Transfer::Rejected, *off)).ok_or(None)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let (off, state) = entry;
        let (left, right) = slice.split_at(off);
        (left, self.end.extract(right, state))
    }
}
