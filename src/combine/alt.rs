use super::*;

#[inline(always)]
pub const fn alt<'i, U, E, A>(alt: A) -> Alternate<'i, U, E, A>
where
    U: ?Sized + Slice,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    Alternate {
        alt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Alternate<'i, U, E, A>
where
    U: ?Sized + Slice,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    alt: A,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Alternatable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_alt(&self) -> Self::Internal;

    fn precede_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, A> Pattern<'i, U, E> for Alternate<'i, U, E, A>
where
    U: ?Sized + Slice,
    E: Situation,
    A: Alternatable<'i, U, E>,
{
    type Captured = A::Captured;
    type Internal = A::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.alt.init_alt()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.alt.precede_alt(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.alt.extract_alt(slice, entry)
    }
}

macro_rules! impl_alternatable_for_tuple {
    ( $Alt:ident, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Alternatable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
        {
            type Captured = $Alt<$($GenN::Captured),+>;
            type Internal = $Alt<$($GenN::Internal),+>;

            #[inline(always)]
            fn init_alt(&self) -> Self::Internal {
                $Alt::Var1(self.0.init())
            }

            #[inline(always)]
            #[allow(irrefutable_let_patterns)]
            fn precede_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use $Alt::*;

                resume_precede! {
                    entry => { $(
                        $LabN: $VarN(_) => [{
                            *entry = $VarN(self.$IdxN.init());
                        }] {
                            let $VarN(state) = entry else { unreachable!() };
                            match self.$IdxN.precede(slice, state, eof) {
                                Ok(len) => return Ok(len),
                                Err(e) => if !e.is_rejected() {
                                    return Err(e);
                                }
                            }
                        }
                    )+ }
                }

                E::raise_reject_at(0)
            }

            #[inline(always)]
            fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use $Alt::*;
                match entry { $(
                    $VarN(state) => $VarN(self.$IdxN.extract(slice, state)),
                )+ }
            }
        }
    } };
}

macro_rules! impl_alternatable_for_tuples {
    (      $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_alternatable_for_tuples! { @
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_alternatable_for_tuple!( [<Alt $Lens1K>], $($LabN ~ [<P $OrdN>] ~ [<Var $OrdN>] ~ $OrdN ~ $IdxN)+ );

        impl_alternatable_for_tuples! { @
            $($Lens1N ~ $LabN ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_alternatable_for_tuples! {
    0  ~ 'p1  ~ 1  ~ 0
    1  ~ 'p2  ~ 2  ~ 1
    2  ~ 'p3  ~ 3  ~ 2
    3  ~ 'p4  ~ 4  ~ 3
    4  ~ 'p5  ~ 5  ~ 4
    5  ~ 'p6  ~ 6  ~ 5
    6  ~ 'p7  ~ 7  ~ 6
    7  ~ 'p8  ~ 8  ~ 7
    8  ~ 'p9  ~ 9  ~ 8
    9  ~ 'p10 ~ 10 ~ 9
    10 ~ 'p11 ~ 11 ~ 10
    11 ~ 'p12 ~ 12 ~ 11
    12 ~ 'p13 ~ 13 ~ 12
    13 ~ 'p14 ~ 14 ~ 13
    14 ~ 'p15 ~ 15 ~ 14
    15 ~ 'p16 ~ 16 ~ 15
    16 ~ 'p17 ~ 17 ~ 16
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn main() {
        let pat = __pat::<str, _, SimpleError>(("0", alt((("b", is_bin..), ("o", is_oct..), ("x", is_hex..)))));
        assert_eq!(pat.full_match("0b101010").unwrap().1, Alt3::Var1(("b", "101010")));
        assert_eq!(pat.full_match("0o234567").unwrap().1, Alt3::Var2(("o", "234567")));
        assert_eq!(pat.full_match("0xabcdef").unwrap().1, Alt3::Var3(("x", "abcdef")));
        assert_eq!(pat.full_match("0x").unwrap_err().length(), 1);
        assert_eq!(pat.full_match("0").unwrap_err().length(), 1);
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
    }
}
