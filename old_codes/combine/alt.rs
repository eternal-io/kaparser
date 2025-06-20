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
        impl<'i, U, E, $($GenN),+> PatternV2<'i, U, E> for Alternate<($($GenN,)+)>
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: PatternV2<'i, U, E>,)+
        {
            type Captured = [<Alt $Len>]<$($GenN::Captured),+>;

            #[inline]
            fn parse_impl(&self, input: &impl Stream<'i, U>) -> Result<(Self::Captured, usize), E> {
                use [<Alt $Len>]::*;
            $(
                match self.alt.$IdxN.parse_impl(input) {
                    Ok((cap, len)) => return Ok(($VarN(cap), len)),
                    Err(e) => if !e.is_rejected() {
                        return Err(e);
                    }
                }
            )+
                E::raise_reject_at(0)
            }
        }
    } };
}

__generate_codes! { impl_alternate_for_tuple ( A ~ Var ) }

//------------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn test_alt() {
//         let pat = opaque_simple(("0", alt((("b", is_bin..), ("o", is_oct..), ("x", is_hex..)))));
//         assert_eq!(pat.fullmatch("0b101010").unwrap().1, Alt3::Var1(("b", "101010")));
//         assert_eq!(pat.fullmatch("0o234567").unwrap().1, Alt3::Var2(("o", "234567")));
//         assert_eq!(pat.fullmatch("0xabcdef").unwrap().1, Alt3::Var3(("x", "abcdef")));
//         assert_eq!(pat.fullmatch("0x").unwrap_err().offset(), 1);
//         assert_eq!(pat.fullmatch("0").unwrap_err().offset(), 1);
//         assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
//     }
// }
