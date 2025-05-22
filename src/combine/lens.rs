use super::*;
use core::mem::MaybeUninit;

#[doc(inline)]
pub use crate::len;

#[inline]
pub const fn lens<'i, U, P, E, const N: usize>(predicate: P) -> Lens<'i, U, P, E, N>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
{
    Lens {
        predicate,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Lens<'i, U, P, E, const N: usize>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
{
    predicate: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, P, E, const N: usize> Pattern<'i, U, E> for Lens<'i, U, P, E, N>
where
    U: ?Sized + Slice + 'i,
    P: Predicate<U::Item>,
    E: Situation,
{
    type Captured = [U::Item; N];
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (m_items, offset) = match slice
            .iter_indices()
            .take(N)
            .enumerate()
            .take_while(|(_ctr, (_off, item))| self.predicate.predicate(item))
            .last()
        {
            None => (0, 0),
            Some((ctr, (off, item))) => (ctr + 1, off + U::len_of(item)),
        };

        if m_items < N {
            return match eof {
                true => E::raise_reject_at(offset),
                false => E::raise_unfulfilled(None),
            };
        }

        Ok(offset)
    }
    #[inline]
    #[allow(unsafe_code)]
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        let mut array: MaybeUninit<[U::Item; N]> = MaybeUninit::uninit();

        for (i, item) in slice.iter().enumerate().take(N) {
            unsafe {
                (&raw mut (*array.as_mut_ptr())[i]).write(item);
            }
        }

        unsafe { array.assume_init() }
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn main() {
        let pat = len!(3, 0x80..0xAA).opaque_simple::<[u8], _>();
        assert!(pat.full_match([0x80, 0x80, 0x80].as_ref()).is_ok());
        assert_eq!(pat.full_match([0x80, 0x80, 0x7F].as_ref()).unwrap_err().offset(), 2);
        assert_eq!(pat.full_match([0x80, 0x7F, 0x80].as_ref()).unwrap_err().offset(), 1);
        assert_eq!(pat.full_match([0x80, 0x80, 0xAA].as_ref()).unwrap_err().offset(), 2);
        assert!(pat.full_match([0x80, 0x80, 0xA9].as_ref()).is_ok());

        let pat = len!(2, ..).opaque_simple::<str, _>();
        assert_eq!(pat.full_match("你好").unwrap(), ['你', '好']);
        assert_eq!(pat.full_match("孬").unwrap_err().offset(), 1 * 3);
    }
}
