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
pub const fn until<U, P>(end: P) -> RangeToInclusive<P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    RangeToInclusive { end }
}

//------------------------------------------------------------------------------

impl<U, P> Pattern2<U> for RangeTo<P>
where
    U: Slice2,
    P: Predicate<U::Item>,
{
    type Captured = (U, Option<U::Item>);
    type Internal = usize;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        match slice
            .split_at(*entry)
            .1
            .iter_indices()
            .find(|(_, item)| self.end.predicate(item))
        {
            Some((off, item)) => {
                *entry += off;
                Some((Transfer::Accepted, *entry + slice.len_of(item)))
            }
            None => {
                *entry = slice.len();
                eof.then_some((Transfer::Accepted, *entry))
            }
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let (left, right) = slice.split_at(entry);
        (left, right.first())
    }
}

//------------------------------------------------------------------------------

impl<U, P> Pattern2<U> for RangeToInclusive<P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    type Captured = (U, P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        (0, self.end.init2())
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (offset, state) = entry;
        for item in slice.split_at(*offset).1.iter() {
            let mut st = self.end.init2();
            let (t, len) = self.end.precede2(slice.split_at(*offset).1, &mut st, eof)?;
            match t {
                Transfer::Rejected => (),
                t => {
                    *state = st;
                    return Some((t, *offset + len));
                }
            }
            *offset += slice.len_of(item);
        }
        eof.then_some((Transfer::Halt, *offset))
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let (off, state) = entry;
        let (left, right) = slice.split_at(off);
        (left, self.end.extract2(right, state))
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
