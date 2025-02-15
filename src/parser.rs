use crate::{common::*, proceed::*};

pub trait SimpleParser<'i, U: ?Sized + Slice> {
    type Captured;

    fn precede(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize>;

    fn matches(&self, slice: &'i U) -> Option<Self::Captured>;
}

impl<'i, U: 'i + ?Sized + Slice, P: Proceed<'i, U>> SimpleParser<'i, U> for P {
    type Captured = P::Captured;

    #[inline(always)]
    fn precede(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize> {
        let mut state = self.init();
        let (t, len) = self.proceed(slice, &mut state, true).expect("internal error");
        if let Transfer::Accepted = t {
            Ok((self.extract(slice, state), len))
        } else {
            Err(len)
        }
    }

    #[inline(always)]
    fn matches(&self, slice: &'i U) -> Option<Self::Captured> {
        self.precede(slice)
            .ok()
            .and_then(|(cap, len)| (len == slice.len()).then_some(cap))
    }
}
