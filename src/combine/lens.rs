#![allow(clippy::let_unit_value)]
use super::*;

#[doc(inline)]
pub use crate::len;

#[inline(always)]
pub const fn lens<T, P, E, const LEN: usize>(predicate: P) -> Lens<T, P, E, LEN>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    E: Situation,
{
    Lens {
        predicate,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Lens<T, P, E, const LEN: usize>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    E: Situation,
{
    predicate: P,
    phantom: PhantomData<(T, E)>,
}

impl<'i, T, P, E, const LEN: usize> Pattern<'i, [T], E> for Lens<T, P, E, LEN>
where
    T: Copy + PartialEq,
    P: Predicate<T>,
    E: Situation,
{
    type Captured = [T; LEN];
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}
    #[inline(always)]
    fn precede(&self, slice: &[T], _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i [T], _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(LEN).0.try_into().unwrap()
    }
}
