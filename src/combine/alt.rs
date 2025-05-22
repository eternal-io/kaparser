use super::*;

#[inline]
pub const fn alt<'i, U, E, A>(alt: A) -> Alternative<'i, U, E, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    Alternative {
        alt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Alternative<'i, U, E, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    alt: A,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Alternatable<'i, U, E>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_alt(&self) -> Self::Internal;

    fn advance_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, A> Pattern<'i, U, E> for Alternative<'i, U, E, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    type Captured = A::Captured;
    type Internal = A::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.alt.init_alt()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.alt.advance_alt(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.alt.extract_alt(slice, entry)
    }
}

macro_rules! impl_alternatable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $VarN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Alternatable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = [<Alt $Len>]<$($GenN::Captured),+>;
            type Internal = [<Alt $Len>]<$($GenN::Internal),+>;

            #[inline]
            fn init_alt(&self) -> Self::Internal {
                [<Alt $Len>]::Var1(self.0.init())
            }

            #[inline]
            #[allow(irrefutable_let_patterns)]
            fn advance_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Alt $Len>]::*;

                __resume_advance! { entry ; $(
                    $VarN(_) => {
                        *entry = $VarN(self.$IdxN.init());
                    } {
                        let $VarN(state) = entry else { unreachable!() };
                        match self.$IdxN.advance(slice, state, eof) {
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
            fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use [<Alt $Len>]::*;
                match entry { $(
                    $VarN(state) => $VarN(self.$IdxN.extract(slice, state)),
                )+ }
            }
        }
    } };
}

__generate_codes! { impl_alternatable_for_tuple ( P ~ Var ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_alt() {
        let pat = seq((
            "0",
            alt((seq(("b", is_bin..)), seq(("o", is_oct..)), seq(("x", is_hex..)))),
        ))
        .opaque_simple();
        assert_eq!(pat.full_match("0b101010").unwrap().1, Alt3::Var1(("b", "101010")));
        assert_eq!(pat.full_match("0o234567").unwrap().1, Alt3::Var2(("o", "234567")));
        assert_eq!(pat.full_match("0xabcdef").unwrap().1, Alt3::Var3(("x", "abcdef")));
        assert_eq!(pat.full_match("0x").unwrap_err().offset(), 1);
        assert_eq!(pat.full_match("0").unwrap_err().offset(), 1);
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
    }
}
