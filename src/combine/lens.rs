#![allow(clippy::let_unit_value)]
use super::*;

#[doc(inline)]
pub use crate::len;

#[inline(always)]
pub const fn lens<T, P, const LEN: usize>(predicate: P) -> Lens<T, P, LEN>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    Lens {
        predicate,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Lens<T, P, const LEN: usize>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    predicate: P,
    phantom: PhantomData<T>,
}

impl<'i, T, P, const LEN: usize> Pattern2<&'i [T]> for Lens<T, P, LEN>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
{
    type Captured = [T; LEN];
    type Internal = ();

    #[inline(always)]
    fn init2(&self) -> Self::Internal {}
    #[inline(always)]
    fn precede2(&self, slice: &'i [T], entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let _ = entry;
        // TODO: unused EOF!!!
        slice.get(..LEN).and_then(|frag| {
            frag.iter()
                .all(|value| self.predicate.predicate(value))
                .then_some((Transfer::Accepted, LEN))
        })
    }
    #[inline(always)]
    fn extract2(&self, slice: &'i [T], entry: Self::Internal) -> Self::Captured {
        let _ = entry;
        slice.split_at(LEN).0.try_into().expect("contract violation")
    }
}
