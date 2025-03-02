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
    fn precede2<E: Situation>(&self, slice: &'i [T], _ntry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        if slice.len() < LEN {
            match eof {
                true => E::raise_reject_at(slice.len()),
                false => E::raise_unfulfilled(Some((LEN - slice.len()).try_into().unwrap())),
            }
        } else {
            for (off, item) in slice[..LEN].iter_indices() {
                if !self.predicate.predicate(&item) {
                    return E::raise_reject_at(off);
                }
            }
            Ok(LEN)
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: &'i [T], _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(LEN).0.try_into().unwrap()
    }
}
