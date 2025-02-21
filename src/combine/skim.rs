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
    fn precede(&self, slice: &str, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        match slice
            .split_at(*entry)
            .1
            .char_indices()
            .find(|(_, ch)| self.end.predicate(ch))
        {
            Some((off, ch)) => {
                *entry += off;
                Some((Transfer::Accepted, *entry + ch.len_utf8()))
            }
            None => {
                *entry = slice.len();
                eof.then_some((Transfer::Accepted, *entry))
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
    T: 'i + Copy + PartialEq,
    P: Predicate<T>,
{
    type Captured = (&'i [T], Option<T>);
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: &[T], entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        match slice
            .split_at(*entry)
            .1
            .iter()
            .enumerate()
            .find(|(_, value)| self.end.predicate(value))
        {
            Some((off, _)) => {
                *entry += off;
                Some((Transfer::Accepted, *entry + 1))
            }
            None => {
                *entry = slice.len();
                eof.then_some((Transfer::Accepted, *entry))
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.iter().next().cloned())
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
    fn precede(&self, slice: &str, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (off, state) = entry;
        for ch in slice.split_at(*off).1.chars() {
            let (t, len) = self.end.precede(slice.split_at(*off).1, state, eof)?;
            match t {
                Transfer::Rejected => (),
                t => return Some((t, len)),
            }
            *off += ch.len_utf8();
        }
        eof.then_some((Transfer::Rejected, *off))
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
    fn precede(&self, slice: &[T], entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (off, state) = entry;
        while *off < slice.len() {
            let (t, len) = self.end.precede(slice.split_at(*off).1, state, eof)?;
            match t {
                Transfer::Rejected => (),
                t => return Some((t, len)),
            }
            *off += 1;
        }
        eof.then_some((Transfer::Rejected, *off))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let (off, state) = entry;
        let (left, right) = slice.split_at(off);
        (left, self.end.extract(right, state))
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn till() {
        assert_eq!({ ..'!' }.full_match("").unwrap(), ("", None));
        assert_eq!({ ..'!' }.full_match("Foo").unwrap(), ("Foo", None));
        assert_eq!({ ..'!' }.full_match("Bar!").unwrap(), ("Bar", Some('!')));
        assert_eq!({ ..'!' }.full_match("Bar!Baz").unwrap_err(), 4);
        assert_eq!({ ..'!' }.parse("Bar!Baz").unwrap(), (("Bar", Some('!')), 4));

        assert_eq!({ ..0 }.full_match(b"").unwrap(), (b"".as_ref(), None));
        assert_eq!({ ..0 }.full_match(b"Foo").unwrap(), (b"Foo".as_ref(), None));
        assert_eq!({ ..0 }.full_match(b"Bar\0").unwrap(), (b"Bar".as_ref(), Some(0)));
        assert_eq!({ ..0 }.full_match(b"Bar\0Baz").unwrap_err(), 4);
        assert_eq!({ ..0 }.parse(b"Bar\0Baz").unwrap(), ((b"Bar".as_ref(), Some(0)), 4));
    }

    #[test]
    fn until() {}
}
