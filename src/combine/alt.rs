use super::*;

#[inline]
pub const fn alt<A>(alt: A) -> Alternate<A> {
    Alternate { alt }
}

//------------------------------------------------------------------------------

pub struct Alternate<A> {
    alt: A,
}

macro_rules! impl_alternate_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $VarN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        #[doc(hidden)]
        impl<'i, U, E, $($GenN),+> Pattern<'i, U, E> for Alternate<($($GenN,)+)>
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = [<Alt $Len>]<$($GenN::Captured),+>;
            type Internal = [<Alt $Len>]<$($GenN::Internal),+>;

            #[inline]
            fn init(&self) -> Self::Internal {
                [<Alt $Len>]::Var1(self.alt.0.init())
            }

            #[inline]
            #[allow(irrefutable_let_patterns)]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Alt $Len>]::*;

                __resume_advance! { entry ; $(
                    $VarN(_) => {
                        *entry = $VarN(self.alt.$IdxN.init());
                    } {
                        let $VarN(state) = entry else { unreachable!() };
                        match self.alt.$IdxN.advance(slice, state, eof) {
                            Ok(len) => return Ok(len),
                            Err(e) => if !e.is_rejected() {
                                return Err(e);
                            }
                        }
                    }
                )+ }

                E::raise_reject_at(0)
            }

            #[inline]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use [<Alt $Len>]::*;
                match entry { $(
                    $VarN(state) => $VarN(self.alt.$IdxN.extract(slice, state)),
                )+ }
            }

            #[inline]
            fn inject_base_off(&self, entry: &mut Self::Internal, base_off: usize) {
                use [<Alt $Len>]::*;
                match entry { $(
                    $VarN(state) => self.alt.$IdxN.inject_base_off(state, base_off),
                )+ }
            }
        }
    } };
}

__generate_codes! { impl_alternate_for_tuple ( A ~ Var ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_alt() {
        let pat = opaque_simple(("0", alt((("b", is_bin..), ("o", is_oct..), ("x", is_hex..)))));
        assert_eq!(pat.fullmatch("0b101010").unwrap().1, Alt3::Var1(("b", "101010")));
        assert_eq!(pat.fullmatch("0o234567").unwrap().1, Alt3::Var2(("o", "234567")));
        assert_eq!(pat.fullmatch("0xabcdef").unwrap().1, Alt3::Var3(("x", "abcdef")));
        assert_eq!(pat.fullmatch("0x").unwrap_err().offset(), 1);
        assert_eq!(pat.fullmatch("0").unwrap_err().offset(), 1);
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
    }
}
