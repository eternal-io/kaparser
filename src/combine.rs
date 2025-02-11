use super::{common::*, *};
use core::marker::PhantomData;

mod alternate;
mod sequence;

pub use {alternate::*, sequence::*};

impl<'i, U: ?Sized + Slice, S: Sequencable<'i, U>> Proceed<'i, U> for Sequence<'i, U, S> {
    type Capture = S::Capture;
    type State = S::State;

    fn proceed(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult {
        self.seq.proceed_seq(slice, entry, eof)
    }

    fn extract(&self, slice: &'i U, entry: Self::State) -> Self::Capture {
        self.seq.extract_seq(slice, entry)
    }
}

impl<'i, U: ?Sized + Slice, A: Alternatable<'i, U>> Proceed<'i, U> for Alternate<'i, U, A> {
    type Capture = A::Capture;
    type State = A::State;

    fn proceed(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult {
        self.alt.proceed_alt(slice, entry, eof)
    }

    fn extract(&self, slice: &'i U, entry: Self::State) -> Self::Capture {
        self.alt.extract_alt(slice, entry)
    }
}

#[test]
pub fn main() {
    // dbg!(seq(("a", "w", is_hexdigit)).proceed("awa holy!", Default::default()));
}
