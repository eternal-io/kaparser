use super::*;

#[inline]
pub const fn com<'i, U, E, C>(com: C) -> Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    Compound {
        com,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    com: C,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Compoundable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_com(&self) -> Self::Internal;

    fn advance_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, C> Pattern<'i, U, E> for Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    type Captured = C::Captured;
    type Internal = C::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.com.init_com()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.com.advance_com(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.com.extract_com(slice, entry)
    }
}

macro_rules! impl_compoundable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Compoundable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = &'i U;
            type Internal = usize;

            #[inline]
            fn init_com(&self) -> Self::Internal {
                0
            }

            #[inline]
            fn advance_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                *entry = 0;
            $( {
                let mut state = self.$IdxN.init();
                match self.$IdxN.advance(slice.split_at(*entry).1, &mut state, eof) {
                    Ok(len) => *entry += len,
                    Err(e) => return e.raise_backtrack(*entry),
                }
            } )+
                Ok(*entry)
            }

            #[inline]
            fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                slice.split_at(entry).0
            }
        }
    } };
}

__generate_codes! { impl_compoundable_for_tuple ( P ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_com() {
        assert_eq!(
            com((is_bin.., is_oct.., is_hex..))
                .opaque_simple()
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            "0123456789abcdefABCDEF"
        );
    }
}
