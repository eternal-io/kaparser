use crate::{common::*, error::*, pattern::*};

pub trait Parser<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;

    fn parse(&self, slice: &mut &'i U) -> ParseResult<Self::Captured, E>;

    #[inline(always)]
    fn full_match(&self, slice: &'i U) -> ParseResult<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_reject_at(slice.len() - n),
        }
    }
}

impl<'i, U, E, P> Parser<'i, U, E> for P
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;

    #[inline(always)]
    fn parse(&self, slice: &mut &'i U) -> ParseResult<Self::Captured, E> {
        let mut state = self.init();
        match self.precede(*slice, &mut state, true) {
            Ok(len) => {
                let (left, right) = slice.split_at(len);
                *slice = right;
                Ok(self.extract(left, state))
            }
            Err(e) => {
                if e.is_unfulfilled() {
                    panic!("implementation: pull after EOF")
                }
                Err(e)
            }
        }
    }
}
