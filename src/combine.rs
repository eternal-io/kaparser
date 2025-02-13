use crate::{common::*, predicate::*, proceed::*};
use core::{marker::PhantomData, mem::MaybeUninit};

pub mod alt;
pub mod cut;
pub mod not;
pub mod reiterate;
pub mod repeat;
pub mod seq;
pub mod take;
pub mod until;

pub trait Pattern<'i, U: ?Sized + Slice> {
    type Captured;

    fn precede(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize>;

    fn matches(&self, slice: &'i U) -> Option<Self::Captured>;
}

impl<'i, U: ?Sized + Slice, P: Proceed<'i, U>> Pattern<'i, U> for P {
    type Captured = P::Captured;

    fn precede(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize> {
        let mut state = self.init();
        match self.proceed(slice, &mut state, true).expect("internal error") {
            Transfer::Accepted(len) => Ok((self.extract(slice, state), len)),
            Transfer::Rejected => Err(0),
            Transfer::Halt(len) => Err(len),
        }
    }

    fn matches(&self, slice: &'i U) -> Option<Self::Captured> {
        self.precede(slice)
            .ok()
            .and_then(|(cap, len)| (len == slice.len()).then_some(cap))
    }
}
