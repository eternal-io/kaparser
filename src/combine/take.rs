use super::*;

#[inline]
pub const fn take<T, P, R>(range: R, predicate: P) -> Take<P, R>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    Take { range, predicate }
}
#[inline]
pub const fn take0more<T, P>(predicate: P) -> Take0More<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    Take0More { predicate }
}
#[inline]
pub const fn take1more<T, P>(predicate: P) -> RangeFrom<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    RangeFrom { start: predicate }
}

//------------------------------------------------------------------------------

pub struct Take<P, R>
where
    R: URangeBounds,
{
    range: R,
    predicate: P,
}

impl<'i, U, P, E, R> Pattern<'i, U, E> for Take<P, R>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
    R: URangeBounds,
{
    type Captured = &'i U;
    type Internal = (usize, usize);

    #[inline]
    fn init(&self) -> Self::Internal {
        (0, 0)
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (offset, times) = entry;
        if let Some((i, (off, item))) = slice
            .split_at(*offset)
            .1
            .iter_indices()
            .enumerate()
            .take_while(|(i, (_, ch))| self.range.unfulfilled(*times + *i) && self.predicate.predicate(ch))
            .last()
        {
            *offset += off + U::len_of(item);
            *times += i + 1;
        }

        match eof {
            true => match self.range.contains(*times) {
                true => Ok(*offset),
                false => E::raise_reject_at(*offset),
            },
            false => match self.range.unfulfilled(*times) {
                true => E::raise_unfulfilled(None),
                false => Ok(*offset),
            },
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry.0).0
    }
}

//------------------------------------------------------------------------------

pub struct Take0More<P> {
    predicate: P,
}

impl<'i, U, P, E> Pattern<'i, U, E> for Take0More<P>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = usize;

    #[inline]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match slice
            .split_at(*entry)
            .1
            .iter_indices()
            .find(|(_, item)| !self.predicate.predicate(item))
        {
            Some((off, _)) => {
                *entry += off;
                Ok(*entry)
            }
            None => {
                *entry = slice.len();
                match eof {
                    true => Ok(*entry),
                    false => E::raise_unfulfilled(None),
                }
            }
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry).0
    }
}

//------------------------------------------------------------------------------

use core::ops::RangeFrom;

impl<'i, U, P, E> Pattern<'i, U, E> for RangeFrom<P>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = usize;

    #[inline]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match slice
            .split_at(*entry)
            .1
            .iter_indices()
            .find(|(_, item)| !self.start.predicate(item))
        {
            Some((off, _)) => {
                *entry += off;
                match *entry > 0 {
                    true => Ok(*entry),
                    false => E::raise_reject_at(0),
                }
            }
            None => {
                *entry = slice.len();
                match eof {
                    true => match *entry > 0 {
                        true => Ok(*entry),
                        false => E::raise_reject_at(0),
                    },
                    false => E::raise_unfulfilled(None),
                }
            }
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry).0
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn main() {
        let pat = impls::opaque_simple(take(1..3, unc::upper));
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("Ａ").unwrap(), "Ａ");
        assert_eq!(pat.full_match("ＡＢ").unwrap(), "ＡＢ");
        assert_eq!(pat.full_match("ＡＢＣ").unwrap_err().offset(), 6);

        let pat = impls::opaque_simple(take(2..=3, is_alpha));
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("a").unwrap_err().offset(), 1);
        assert_eq!(pat.full_match("ab").unwrap(), "ab");
        assert_eq!(pat.full_match("abc").unwrap(), "abc");
        assert_eq!(pat.full_match("abcd").unwrap_err().offset(), 3);

        let pat = impls::opaque_simple(take(4, is_alpha));
        assert_eq!(pat.full_match("abc").unwrap_err().offset(), 3);
        assert_eq!(pat.full_match("abcd").unwrap(), "abcd");
        assert_eq!(pat.full_match("abcde").unwrap_err().offset(), 4);

        let pat = impls::opaque_simple::<[u8], _>(take(4, not(0)));
        assert_eq!(pat.full_match(b"abc\0").unwrap_err().offset(), 3);
        assert_eq!(pat.full_match(b"abc\n").unwrap(), b"abc\n");

        let pat = impls::opaque_simple::<[u8], _>(take(2..=3, not(0)));
        assert_eq!(pat.parse(&mut b"a\0".as_ref()).unwrap_err().offset(), 1);
        assert_eq!(pat.parse(&mut b"ab\0d".as_ref()).unwrap(), b"ab".as_ref());
        assert_eq!(pat.parse(&mut b"ab\nd".as_ref()).unwrap(), b"ab\n".as_ref());
    }

    #[test]
    fn one_more() {
        let pat = impls::opaque_simple(is_dec..);
        assert_eq!(pat.full_match("!").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("0123!").unwrap_err().offset(), 4);
        assert_eq!(pat.full_match("7890").unwrap(), "7890");
        assert_eq!(pat.parse(&mut "!").unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut "0123!").unwrap(), "0123");
        assert_eq!(pat.parse(&mut "7890").unwrap(), "7890");

        let pat = impls::opaque_simple::<[u8], _>(not(0)..);
        assert_eq!(pat.full_match(b"\0").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match(b"0123\0").unwrap_err().offset(), 4);
        assert_eq!(pat.full_match(b"7890").unwrap(), b"7890");
        assert_eq!(pat.parse(&mut b"\0".as_ref()).unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut b"0123\0".as_ref()).unwrap(), b"0123");
        assert_eq!(pat.parse(&mut b"7890".as_ref()).unwrap(), b"7890");
    }
}
