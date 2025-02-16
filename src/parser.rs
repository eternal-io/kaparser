#[cfg(feature = "std")]
extern crate std;

use crate::{common::*, pattern::*};

pub trait SimpleParser<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;

    fn parse(&self, slice: &'i U) -> Result<Self::Captured, usize>;

    fn parse_partial(&self, slice: &'i U) -> Result<(Self::Captured, &'i U), usize>;
}

impl<'i, U, P> SimpleParser<'i, U> for P
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    type Captured = P::Captured;

    #[inline(always)]
    fn parse(&self, slice: &'i U) -> Result<Self::Captured, usize> {
        self.parse_partial(slice)
            .and_then(|(cap, rest)| rest.is_empty().then_some(cap).ok_or(rest.len()))
    }

    #[inline(always)]
    fn parse_partial(&self, slice: &'i U) -> Result<(Self::Captured, &'i U), usize> {
        let mut state = self.init();
        let (t, len) = self.precede(slice, &mut state, true).expect("internal error");
        if let Transfer::Accepted = t {
            Ok((self.extract(slice, state), slice.split_at(len).1))
        } else {
            Err(len)
        }
    }
}
