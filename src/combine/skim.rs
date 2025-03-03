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
pub const fn until<U, E, P>(end: P) -> RangeToInclusive<P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
{
    RangeToInclusive { end }
}

//------------------------------------------------------------------------------

impl<U, E, P> Pattern<U, E> for RangeTo<P>
where
    U: Slice,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = (U, Option<U::Item>);
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
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
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.first())
    }
}

//------------------------------------------------------------------------------

impl<U, E, P> Pattern<U, E> for RangeToInclusive<P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
{
    type Captured = (U, P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.end.init())
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
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
                        return Err(e);
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
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let (off, state) = entry;
        let (left, right) = slice.split_at(off);
        (left, self.end.extract(right, state))
    }
}

//------------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn till() {
//         assert_eq!({ ..'🔥' }.full_match("").unwrap(), ("", None));
//         assert_eq!({ ..'🔥' }.full_match("Foo").unwrap(), ("Foo", None));
//         assert_eq!({ ..'🔥' }.full_match("Bar🔥").unwrap(), ("Bar", Some('🔥')));
//         assert_eq!({ ..'🔥' }.full_match("Bar🔥Baz").unwrap_err(), 7);
//         assert_eq!({ ..'🔥' }.parse("Bar🔥Baz").unwrap(), (("Bar", Some('🔥')), 7));

//         assert_eq!({ ..0 }.full_match(b"").unwrap(), (b"".as_ref(), None));
//         assert_eq!({ ..0 }.full_match(b"Foo").unwrap(), (b"Foo".as_ref(), None));
//         assert_eq!({ ..0 }.full_match(b"Bar\0").unwrap(), (b"Bar".as_ref(), Some(0)));
//         assert_eq!({ ..0 }.full_match(b"Bar\0Baz").unwrap_err(), 4);
//         assert_eq!({ ..0 }.parse(b"Bar\0Baz").unwrap(), ((b"Bar".as_ref(), Some(0)), 4));
//     }

//     #[test]
//     fn until() {
//         assert_eq!({ ..="🚧" }.full_match("🚧").unwrap(), ("", "🚧"));
//         assert_eq!({ ..="🚧" }.full_match("FooBar🚧").unwrap(), ("FooBar", "🚧"));
//         assert_eq!({ ..=[0] }.full_match(b"Quinn\0").unwrap(), (b"Quinn".as_ref(), 0));

//         // The following is feature.
//         assert_eq!({ ..="" }.parse("").unwrap_err(), 0);
//         assert_eq!({ ..="" }.parse("❓").unwrap(), (("", ""), 0));
//         assert_eq!({ ..=[].as_ref() }.parse(b"").unwrap_err(), 0);
//         assert_eq!(
//             { ..=[].as_ref() }.parse(b"??").unwrap(),
//             ((b"".as_ref(), b"".as_ref()), 0)
//         );
//     }
// }
