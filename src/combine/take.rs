use super::*;
use core::ops::RangeFrom;

#[inline(always)]
pub const fn when<T, P>(start: P) -> RangeFrom<P>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    RangeFrom { start }
}

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

impl<U, P, R> Pattern2<U> for Take<U::Item, P, R>
where
    U: Slice2,
    P: Predicate<U::Item>,
    R: URangeBounds,
{
    type Captured = U;
    type Internal = (usize, usize);

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        (0, 0)
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
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

        Some(match eof {
            true => match self.range.contains(*times) {
                true => (Transfer::Accepted, *offset),
                false => (Transfer::Rejected, *offset),
            },
            false => match self.range.unfulfilled(*times) {
                true => return None,
                false => (Transfer::Accepted, *offset),
            },
        })
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry.0).0
    }
}

//------------------------------------------------------------------------------

impl<U, P> Pattern2<U> for RangeFrom<P>
where
    U: Slice2,
    P: Predicate<U::Item>,
{
    type Captured = U;
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
            .find(|(_, item)| !self.start.predicate(item))
        {
            Some((off, _)) => {
                *entry += off;
                Some(Transfer::perhaps((*entry > 0).then_some(*entry).ok_or(0)))
            }
            None => {
                *entry = slice.len();
                eof.then_some(Transfer::perhaps((*entry > 0).then_some(*entry).ok_or(0)))
            }
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry).0
    }
}

//------------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn times() {
//         assert_eq!(take(1..3, unc::upper).full_match("").unwrap_err(), 0);
//         assert_eq!(take(1..3, unc::upper).full_match("Ａ").unwrap(), "Ａ");
//         assert_eq!(take(1..3, unc::upper).full_match("ＡＢ").unwrap(), "ＡＢ");
//         assert_eq!(take(1..3, unc::upper).full_match("ＡＢＣ").unwrap_err(), 6);

//         assert_eq!(take(2..=3, is_alpha).full_match("").unwrap_err(), 0);
//         assert_eq!(take(2..=3, is_alpha).full_match("a").unwrap_err(), 1);
//         assert_eq!(take(2..=3, is_alpha).full_match("ab").unwrap(), "ab");
//         assert_eq!(take(2..=3, is_alpha).full_match("abc").unwrap(), "abc");
//         assert_eq!(take(2..=3, is_alpha).full_match("abcd").unwrap_err(), 3);

//         assert_eq!(take(4, is_alpha).full_match("abc").unwrap_err(), 3);
//         assert_eq!(take(4, is_alpha).full_match("abcd").unwrap(), "abcd");
//         assert_eq!(take(4, is_alpha).full_match("abcde").unwrap_err(), 4);

//         assert_eq!(take(4, not(0)).full_match(b"abc\0").unwrap_err(), 3);
//         assert_eq!(take(4, not(0)).full_match(b"abc\n").unwrap(), b"abc\n");

//         assert_eq!(take(2..=3, not(0)).parse(b"a\0").unwrap_err(), 1);
//         assert_eq!(take(2..=3, not(0)).parse(b"ab\0d").unwrap(), (b"ab".as_ref(), 2));
//         assert_eq!(take(2..=3, not(0)).parse(b"ab\nd").unwrap(), (b"ab\n".as_ref(), 3));
//     }

//     #[test]
//     fn one_more() {
//         assert_eq!({ is_dec.. }.full_match("!").unwrap_err(), 0);
//         assert_eq!({ is_dec.. }.full_match("0123!").unwrap_err(), 4);
//         assert_eq!({ is_dec.. }.full_match("7890").unwrap(), "7890");
//         assert_eq!({ is_dec.. }.parse("!").unwrap_err(), 0);
//         assert_eq!({ is_dec.. }.parse("0123!").unwrap(), ("0123", 4));
//         assert_eq!({ is_dec.. }.parse("7890").unwrap(), ("7890", 4));

//         assert_eq!({ not(0).. }.full_match(b"\0").unwrap_err(), 0);
//         assert_eq!({ not(0).. }.full_match(b"0123\0").unwrap_err(), 4);
//         assert_eq!({ not(0).. }.full_match(b"7890").unwrap(), b"7890");
//         assert_eq!({ not(0).. }.parse(b"\0").unwrap_err(), 0);
//         assert_eq!({ not(0).. }.parse(b"0123\0").unwrap(), (b"0123".as_ref(), 4));
//         assert_eq!({ not(0).. }.parse(b"7890").unwrap(), (b"7890".as_ref(), 4));
//     }

//     // #[test]
//     // #[cfg(feature = "std")]
//     // fn streaming() {
//     //     let s = "EFAB6251-2b3e-4395-bfc0-370e268935d1";
//     //     let pat = seq((
//     //         take(8, is_hex),
//     //         "-",
//     //         take(4, is_hex),
//     //         "-",
//     //         take(4, is_hex),
//     //         "-",
//     //         take(4, is_hex),
//     //         "-",
//     //         is_hex..,
//     //     ));

//     //     let mut par = Parser::from_reader_in_str_with_capacity(s.as_bytes(), 0);

//     //     assert_eq!(
//     //         par.next_str(pat).unwrap(),
//     //         ("EFAB6251", "-", "2b3e", "-", "4395", "-", "bfc0", "-", "370e268935d1")
//     //     );

//     //     assert!(par.exhausted());
//     // }
// }
