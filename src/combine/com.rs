use super::*;

#[inline]
pub const fn com<C>(com: C) -> Compound<C> {
    Compound { com }
}

//------------------------------------------------------------------------------

pub struct Compound<C> {
    com: C,
}

macro_rules! impl_compound_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        #[doc(hidden)]
        impl<'i, U, E, $($GenN),+> Pattern<'i, U, E> for Compound<($($GenN,)+)>
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = &'i U;
            type Internal = usize;

            #[inline]
            fn init(&self) -> Self::Internal {
                0
            }

            #[inline]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                *entry = 0;
            $( {
                let mut state = self.com.$IdxN.init();
                match self.com.$IdxN.advance(slice.after(*entry), &mut state, eof) {
                    Ok(len) => *entry += len,
                    Err(e) => return e.raise_backtrack(*entry),
                }
            } )+
                Ok(*entry)
            }

            #[inline]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                slice.before(entry)
            }
        }
    } };
}

__generate_codes! { impl_compound_for_tuple ( C ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_com() {
        assert_eq!(
            opaque_simple(com((is_bin.., is_oct.., is_hex..)))
                .fullmatch("0123456789abcdefABCDEF")
                .unwrap(),
            "0123456789abcdefABCDEF"
        );
    }
}
