use super::*;
use core::ops::{RangeTo, RangeToInclusive};

#[inline(always)]
pub const fn till<T, P>(end: P) -> RangeTo<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    RangeTo { end }
}

#[inline(always)]
pub const fn until<'i, U, E, P>(end: P) -> RangeToInclusive<P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    RangeToInclusive { end }
}

//------------------------------------------------------------------------------

impl<'i, U, E, P> Pattern<'i, U, E> for RangeTo<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = (&'i U, Option<U::Item>);
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
            .find(|(_, item)| self.end.predicate(item))
        {
            Some((off, item)) => {
                *entry += off;
                Ok(*entry + slice.len_of(item))
            }
            None => {
                *entry = slice.len();
                eof.then_some(*entry).ok_or(E::unfulfilled(None))
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.first())
    }
}

//------------------------------------------------------------------------------

impl<'i, U, E, P> Pattern<'i, U, E> for RangeToInclusive<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = (&'i U, P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.end.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (offset, state) = entry;
        for item in slice.split_at(*offset).1.iter() {
            let mut st = self.end.init();
            let res = self.end.precede(slice.split_at(*offset).1, &mut st, eof);
            match res {
                Ok(len) => {
                    *state = st;
                    return Ok(*offset + len);
                }
                Err(e) => {
                    if !e.is_rejected() {
                        return e.raise_backtrack(*offset);
                    }
                }
            }
            *offset += slice.len_of(item);
        }

        match eof {
            true => E::raise_halt_at(*offset),
            false => E::raise_unfulfilled(None),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
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
        let pat = __pat::<str, _, ParseError>(..'🔥');
        assert_eq!(pat.full_match("").unwrap(), ("", None));
        assert_eq!(pat.full_match("Foo").unwrap(), ("Foo", None));
        assert_eq!(pat.full_match("Bar🔥").unwrap(), ("Bar", Some('🔥')));
        assert_eq!(pat.full_match("Bar🔥Baz").unwrap_err().length(), 7);
        assert_eq!(pat.parse(&mut "Bar🔥Baz").unwrap(), ("Bar", Some('🔥')));
    }

    #[test]
    fn until() {
        let pat = __pat::<str, _, ParseError>(..="🚧");
        assert_eq!(pat.full_match("🚧").unwrap(), ("", "🚧"));
        assert_eq!(pat.full_match("FooBar🚧").unwrap(), ("FooBar", "🚧"));

        let pat = __pat::<[u8], _, ParseError>(..=[0]);
        assert_eq!(pat.full_match(b"Quinn\0").unwrap(), (b"Quinn".as_ref(), 0));

        /* The following is feature. */

        let pat = __pat::<str, _, ParseError>(..="");
        assert_eq!(pat.parse(&mut "").unwrap_err().length(), 0);
        assert_eq!(pat.parse(&mut "❓").unwrap(), ("", ""));

        let pat = __pat::<[u8], _, ParseError>(..=[].as_ref());
        assert_eq!(pat.parse(&mut b"".as_ref()).unwrap_err().length(), 0);
        assert_eq!(pat.parse(&mut b"??".as_ref()).unwrap(), (b"".as_ref(), b"".as_ref()));
    }
}
