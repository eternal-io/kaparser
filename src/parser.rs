use crate::{common::*, precede::*};

pub trait SimpleParser<'i, U: ?Sized + Slice> {
    type Captured;

    fn parse_partial(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize>;

    fn parse(&self, slice: &'i U) -> Option<Self::Captured>;
}

impl<'i, U: 'i + ?Sized + Slice, P: Precede<'i, U>> SimpleParser<'i, U> for P {
    type Captured = P::Captured;

    #[inline(always)]
    fn parse_partial(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize> {
        let mut state = self.init();
        let (t, len) = self.precede(slice, &mut state, true).expect("internal error");
        if let Transfer::Accepted = t {
            Ok((self.extract(slice, state), len))
        } else {
            Err(len)
        }
    }

    #[inline(always)]
    fn parse(&self, slice: &'i U) -> Option<Self::Captured> {
        self.parse_partial(slice)
            .ok()
            .and_then(|(cap, len)| (len == slice.len()).then_some(cap))
    }
}
