use super::*;
use core::ops::RangeFrom;

#[inline(always)]
pub const fn take<T, P: Predicate<T>, R: URangeBounds>(range: R, predicate: P) -> Take<T, P, R> {
    Take {
        range,
        predicate,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Take<T, P: Predicate<T>, R: URangeBounds> {
    range: R,
    predicate: P,
    phantom: PhantomData<T>,
}

impl<'i, P: Predicate<char>, R: URangeBounds> Proceed<'i, str> for Take<char, P, R> {
    type Captured = &'i str;
    type Internal = (usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1)
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let (tot_off, times) = entry;
        if let Some((i, (off, ch))) = slice
            .split_at(*tot_off)
            .1
            .char_indices()
            .enumerate()
            .take_while(|(i, (_, ch))| self.range.unsaturated(*times + *i) && self.predicate.predicate(ch))
            .last()
        {
            *tot_off += off + ch.len_utf8();
            *times += i;
        }

        match eof {
            true => match self.range.contains(*times) {
                true => Ok(Transfer::Accepted(*tot_off)),
                false => Ok(Transfer::Rejected),
            },
            false => match !self.range.unsaturated(*times) {
                true => Ok(Transfer::Accepted(*tot_off)),
                false => Err(None),
            },
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        &slice[..entry.0]
    }
}

//------------------------------------------------------------------------------

impl<'i, P: Predicate<char>> Proceed<'i, str> for RangeFrom<P> {
    type Captured = &'i str;
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        match slice
            .split_at(*entry)
            .1
            .char_indices()
            .find(|(_, ch)| !self.start.predicate(ch))
        {
            Some((off, _)) => Ok(Transfer::Accepted(*entry + off)),
            None => {
                *entry = slice.len();
                eof.then_some(Transfer::Accepted(*entry)).ok_or(None)
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        &slice[..entry]
    }
}

impl<'i, T: 'i, P: Predicate<T>> Proceed<'i, [T]> for RangeFrom<P> {
    type Captured = &'i [T];
    type Internal = usize;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        0
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        match slice
            .split_at(*entry)
            .1
            .iter()
            .enumerate()
            .find(|(_, value)| !self.start.predicate(value))
        {
            Some((off, _)) => Ok(Transfer::Accepted(*entry + off)),
            None => {
                *entry = slice.len();
                eof.then_some(Transfer::Accepted(*entry)).ok_or(None)
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        &slice[..entry]
    }
}
