use super::*;
use core::ops::{RangeTo, RangeToInclusive};

/// The terminator is optional but also consumed.
#[inline]
pub const fn till<T, P>(end: P) -> RangeTo<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    RangeTo { end }
}
/// The terminator is required and also consumed.
#[inline]
pub const fn until<'i, U, E, P>(end: P) -> RangeToInclusive<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    RangeToInclusive { end }
}

#[inline]
pub const fn xtill<U, X>(needle: X) -> FastTill<X>
where
    U: ?Sized + ThinSlice,
    X: Needlable<U>,
{
    FastTill { needle }
}
#[inline]
pub const fn xuntil<U>(needle: &U) -> FastUntil<U>
where
    U: ?Sized + ThinSlice,
{
    FastUntil { needle }
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
            .find(|(_, item)| self.end.predicate(item))
        {
            Some((off, item)) => {
                *entry += off;
                Ok(*entry + U::len_of(item))
            }
            None => {
                *entry = slice.len();
                eof.then_some(*entry).ok_or(E::unfulfilled(None))
            }
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (before, after) = slice.split_at(entry);

        (before, after.first())
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

    #[inline]
    fn init(&self) -> Self::Internal {
        (0, self.end.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (offset, state) = entry;
        for item in slice.split_at(*offset).1.iter() {
            let mut st = self.end.init();
            let res = self.end.advance(slice.split_at(*offset).1, &mut st, eof);
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
            *offset += U::len_of(item);
        }

        match eof {
            true => E::raise_halt_at(*offset),
            false => E::raise_unfulfilled(None),
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (offset, state) = entry;
        let (before, after) = slice.split_at(offset);

        (before, self.end.extract(after, state))
    }
}

//------------------------------------------------------------------------------

pub struct FastTill<X> {
    needle: X,
}

impl<'i, U, X, E> Pattern<'i, U, E> for FastTill<X>
where
    U: ?Sized + ThinSlice + 'i,
    X: Needlable<U>,
    E: Situation,
{
    type Captured = (&'i U, Option<U::Item>);
    type Internal = usize;

    #[inline]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        todo!()
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (before, after) = slice.split_at(entry);

        (before, after.first())
    }
}

//------------------------------------------------------------------------------

pub struct FastUntil<'s, U>
where
    U: ?Sized + ThinSlice + 's,
{
    needle: &'s U,
}

impl<'i, U, E> Pattern<'i, U, E> for FastUntil<'_, U>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = usize;

    #[inline]
    fn init(&self) -> Self::Internal {
        todo!()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        todo!()
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.before(entry)
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn till() {
        let pat = impls::opaque_simple(..'🔥');
        assert_eq!(pat.full_match("").unwrap(), ("", None));
        assert_eq!(pat.full_match("Foo").unwrap(), ("Foo", None));
        assert_eq!(pat.full_match("Bar🔥").unwrap(), ("Bar", Some('🔥')));
        assert_eq!(pat.full_match("Bar🔥Baz").unwrap_err().offset(), 7);
        assert_eq!(pat.parse(&mut "Bar🔥Baz").unwrap(), ("Bar", Some('🔥')));
    }

    #[test]
    fn until() {
        let pat = impls::opaque_simple(..="🚧");
        assert_eq!(pat.full_match("🚧").unwrap(), ("", "🚧"));
        assert_eq!(pat.full_match("FooBar🚧").unwrap(), ("FooBar", "🚧"));

        let pat = impls::opaque_simple::<[u8], _>(..=[0]);
        assert_eq!(pat.full_match(b"Quinn\0").unwrap(), (b"Quinn".as_ref(), 0));

        /* The following is feature. */

        let pat = impls::opaque_simple(..="");
        assert_eq!(pat.parse(&mut "").unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut "❓").unwrap(), ("", ""));

        let pat = impls::opaque_simple::<[u8], _>(..=[].as_ref());
        assert_eq!(pat.parse(&mut b"".as_ref()).unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut b"??".as_ref()).unwrap(), (b"".as_ref(), b"".as_ref()));
    }
}
