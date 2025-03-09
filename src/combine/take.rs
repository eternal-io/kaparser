use super::*;
use core::ops::RangeFrom;

#[inline(always)]
pub const fn take<T, P, R>(range: R, predicate: P) -> Take<T, P, R>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    Take {
        range,
        predicate,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn take0<T, P>(predicate: P) -> Take0<T, P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    Take0 {
        predicate,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn take1<T, P>(predicate: P) -> RangeFrom<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    RangeFrom { start: predicate }
}

//------------------------------------------------------------------------------

pub struct Take<T, P, R>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    range: R,
    predicate: P,
    phantom: PhantomData<T>,
}

impl<'i, U, E, P, R> Pattern<'i, U, E> for Take<U::Item, P, R>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Predicate<U::Item>,
    R: URangeBounds,
{
    type Captured = &'i U;
    type Internal = (usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 0)
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (offset, times) = entry;
        if let Some((i, (off, item))) = slice
            .split_at(*offset)
            .1
            .iter_indices()
            .enumerate()
            .take_while(|(i, (_, ch))| self.range.unfulfilled(*times + *i) && self.predicate.predicate(ch))
            .last()
        {
            *offset += off + slice.len_of(item);
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
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry.0).0
    }
}

//------------------------------------------------------------------------------

pub struct Take0<T, P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    predicate: P,
    phantom: PhantomData<T>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Take0<U::Item, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = &'i U;
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry).0
    }
}

//------------------------------------------------------------------------------

impl<'i, U, E, P> Pattern<'i, U, E> for RangeFrom<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = &'i U;
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    #[inline(always)]
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
        let pat = opaque::<str, _, SimpleError>(take(1..3, unc::upper));
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("Ａ").unwrap(), "Ａ");
        assert_eq!(pat.full_match("ＡＢ").unwrap(), "ＡＢ");
        assert_eq!(pat.full_match("ＡＢＣ").unwrap_err().length(), 6);

        let pat = opaque::<str, _, SimpleError>(take(2..=3, is_alpha));
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("a").unwrap_err().length(), 1);
        assert_eq!(pat.full_match("ab").unwrap(), "ab");
        assert_eq!(pat.full_match("abc").unwrap(), "abc");
        assert_eq!(pat.full_match("abcd").unwrap_err().length(), 3);

        let pat = opaque::<str, _, SimpleError>(take(4, is_alpha));
        assert_eq!(pat.full_match("abc").unwrap_err().length(), 3);
        assert_eq!(pat.full_match("abcd").unwrap(), "abcd");
        assert_eq!(pat.full_match("abcde").unwrap_err().length(), 4);

        let pat = opaque::<[u8], _, SimpleError>(take(4, not(0)));
        assert_eq!(pat.full_match(b"abc\0").unwrap_err().length(), 3);
        assert_eq!(pat.full_match(b"abc\n").unwrap(), b"abc\n");

        let pat = opaque::<[u8], _, SimpleError>(take(2..=3, not(0)));
        assert_eq!(pat.parse(&mut b"a\0".as_ref()).unwrap_err().length(), 1);
        assert_eq!(pat.parse(&mut b"ab\0d".as_ref()).unwrap(), b"ab".as_ref());
        assert_eq!(pat.parse(&mut b"ab\nd".as_ref()).unwrap(), b"ab\n".as_ref());
    }

    #[test]
    fn one_more() {
        let pat = opaque::<str, _, SimpleError>(is_dec..);
        assert_eq!(pat.full_match("!").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("0123!").unwrap_err().length(), 4);
        assert_eq!(pat.full_match("7890").unwrap(), "7890");
        assert_eq!(pat.parse(&mut "!").unwrap_err().length(), 0);
        assert_eq!(pat.parse(&mut "0123!").unwrap(), "0123");
        assert_eq!(pat.parse(&mut "7890").unwrap(), "7890");

        let pat = opaque::<[u8], _, SimpleError>(not(0)..);
        assert_eq!(pat.full_match(b"\0").unwrap_err().length(), 0);
        assert_eq!(pat.full_match(b"0123\0").unwrap_err().length(), 4);
        assert_eq!(pat.full_match(b"7890").unwrap(), b"7890");
        assert_eq!(pat.parse(&mut b"\0".as_ref()).unwrap_err().length(), 0);
        assert_eq!(pat.parse(&mut b"0123\0".as_ref()).unwrap(), b"0123");
        assert_eq!(pat.parse(&mut b"7890".as_ref()).unwrap(), b"7890");
    }

    #[test]
    #[cfg(feature = "std")]
    fn streaming() -> ProviderResult<()> {
        let s = "EFAB6251-2b3e-4395-bfc0-370e268935d1";
        let pat = (take(8, is_hex), rep!(3, ("-", take(4, is_hex))), ("-", is_hex..));
        let mut prv = Provider::from_reader_in_str_with_capacity(s.as_bytes(), 0);

        assert_eq!(
            prv.next_str(&pat)?,
            (
                "EFAB6251",
                [("-", "2b3e"), ("-", "4395"), ("-", "bfc0")],
                ("-", "370e268935d1"),
            )
        );

        assert!(prv.exhausted());

        Ok(())
    }
}
