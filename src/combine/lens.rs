#![allow(clippy::let_unit_value)]
use super::*;

#[doc(inline)]
pub use crate::len;

#[inline(always)]
pub const fn lens<'i, T, P, const LEN: usize>(predicate: P) -> Lens<'i, T, P, LEN>
where
    T: 'i + Copy + PartialEq,
    P: Predicate<T>,
{
    Lens {
        predicate,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Lens<'i, T, P, const LEN: usize>
where
    T: 'i + Copy + PartialEq,
    P: Predicate<T>,
{
    predicate: P,
    phantom: PhantomData<&'i T>,
}

impl<'i, T, P, const LEN: usize> Pattern<'i, [T]> for Lens<'i, T, P, LEN>
where
    T: 'i + Copy + PartialEq,
    P: Predicate<T>,
{
    type Captured = [T; LEN];
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}
    #[inline(always)]
    fn precede(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let _ = entry;
        slice
            .get(..LEN)
            .and_then(|frag| {
                frag.iter()
                    .all(|value| self.predicate.predicate(value))
                    .then_some((Transfer::Accepted, LEN))
            })
            .ok_or(
                eof.then(|| Some((LEN - slice.len()).try_into().unwrap()))
                    .unwrap_or(None),
            )
    }
    #[inline(always)]
    fn extract(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.split_at(LEN).0.try_into().expect("contract violation")
    }
}
