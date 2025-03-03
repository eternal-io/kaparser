use crate::{common::*, error::*, pattern::*};

pub trait Parser<U, E>
where
    U: Slice2,
    E: Situation,
{
    type Captured;

    fn parse(&self, slice: &mut U) -> ParseResult<Self::Captured, E>;

    #[inline(always)]
    fn full_match(&self, slice: U) -> ParseResult<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_reject_at(slice.len() - n),
        }
    }
}

impl<U, E, P> Parser<U, E> for P
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U>,
{
    type Captured = P::Captured;

    #[inline(always)]
    fn parse(&self, slice: &mut U) -> ParseResult<Self::Captured, E> {
        let mut state = self.init2();
        match self.precede2::<E>(*slice, &mut state, true) {
            Ok(len) => {
                let (left, right) = slice.split_at(len);
                *slice = right;
                Ok(self.extract2(left, state))
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
