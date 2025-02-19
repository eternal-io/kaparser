use super::*;
use core::ops::RangeFrom;

// TODO: 把 RangeFrom 的行为改成 take(1..) ！！！

#[inline(always)]
pub const fn take<'i, T, P, R>(range: R, predicate: P) -> Take<'i, T, P, R>
where
    T: 'i + PartialEq,
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

pub struct Take<'i, T, P, R>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    range: R,
    predicate: P,
    phantom: PhantomData<&'i T>,
}

impl<'i, P, R> Pattern<'i, str> for Take<'i, char, P, R>
where
    P: Predicate<char>,
    R: URangeBounds,
{
    type Captured = &'i str;
    type Internal = (usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1)
    }
    #[inline(always)]
    fn precede(&self, slice: &str, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (offset, times) = entry;
        if let Some((i, (off, ch))) = slice
            .split_at(*offset)
            .1
            .char_indices()
            .enumerate()
            .take_while(|(i, (_, ch))| self.range.unfulfilled(*times + *i) && self.predicate.predicate(ch))
            .last()
        {
            *offset += off + ch.len_utf8();
            *times += i;
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
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        &slice[..entry.0]
    }
}

impl<'i, T, P, R> Pattern<'i, [T]> for Take<'i, T, P, R>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    type Captured = &'i [T];
    type Internal = (usize, usize); // TODO: 对于 [T] 只要一个 usize 就好了！

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1)
    }
    #[inline(always)]
    fn precede(&self, slice: &[T], entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (offset, times) = entry;
        if let Some((i, _)) = slice
            .split_at(*offset)
            .1
            .iter()
            .enumerate()
            .take_while(|(i, value)| self.range.unfulfilled(*times + *i) && self.predicate.predicate(value))
            .last()
        {
            *offset += i + 1;
            *times += i;
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
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        &slice[..entry.0]
    }
}

//------------------------------------------------------------------------------

impl<'i, P> Pattern<'i, str> for RangeFrom<P>
where
    P: Predicate<char>,
{
    type Captured = &'i str;
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
            .find(|(_, ch)| !self.start.predicate(ch))
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
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        &slice[..entry]
    }
}

impl<'i, T, P> Pattern<'i, [T]> for RangeFrom<P>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
{
    type Captured = &'i [T];
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
            .find(|(_, value)| !self.start.predicate(value))
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
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        &slice[..entry]
    }
}
