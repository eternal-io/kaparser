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
    fn precede2(&self, slice: &'i [T], _ntry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        if slice.len() < LEN {
            eof.then_some((Transfer::Rejected, slice.len()))
        } else {
            slice[..LEN]
                .iter()
                .all(|item| self.predicate.predicate(item))
                .then_some((Transfer::Accepted, LEN))
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: &'i [T], _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(LEN).0.try_into().expect("contract violation")
    }
}
