use super::*;
use core::ops::RangeFrom;

#[inline(always)]
pub const fn take<T, P, R>(range: R, predicate: P) -> Take<T, P, R>
where
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
    P: Predicate<T>,
    R: URangeBounds,
{
    range: R,
    predicate: P,
    phantom: PhantomData<T>,
}

impl<'i, P, R> Precede<'i, str> for Take<char, P, R>
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
    fn precede(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
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

        Ok(match eof {
            true => match self.range.contains(*times) {
                true => (Transfer::Accepted, *offset),
                false => (Transfer::Rejected, *offset),
            },
            false => match self.range.unfulfilled(*times) {
                true => return Err(None),
                false => (Transfer::Accepted, *offset),
            },
        })
    }
    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        &slice[..entry.0]
    }
}

impl<'i, T, P, R> Precede<'i, [T]> for Take<T, P, R>
where
    T: 'i + PartialEq,
    P: Predicate<T>,
    R: URangeBounds,
{
    type Captured = &'i [T];
    type Internal = (usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1)
    }
    #[inline(always)]
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
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

        Ok(match eof {
            true => match self.range.contains(*times) {
                true => (Transfer::Accepted, *offset),
                false => (Transfer::Rejected, *offset),
            },
            false => match self.range.unfulfilled(*times) {
                true => return Err(None),
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

impl<'i, P> Precede<'i, str> for RangeFrom<P>
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
    fn precede(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        match slice
            .split_at(*entry)
            .1
            .char_indices()
            .find(|(_, ch)| !self.start.predicate(ch))
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
        &slice[..entry]
    }
}

impl<'i, T, P> Precede<'i, [T]> for RangeFrom<P>
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
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        match slice
            .split_at(*entry)
            .1
            .iter()
            .enumerate()
            .find(|(_, value)| !self.start.predicate(value))
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
        &slice[..entry]
    }
}
