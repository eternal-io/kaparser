use super::*;

#[inline(always)]
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

    fn precede_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

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

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.com.init_com()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.com.precede_com(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.com.extract_com(slice, entry)
    }
}

macro_rules! impl_compoundable_for_tuple {
    ( $Alt:ident, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Compoundable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = &'i U;
            type Internal = (usize, $Alt<$($GenN::Internal),+>);

            #[inline(always)]
            fn init_com(&self) -> Self::Internal {
                (0, $Alt::Var1(self.0.init()))
            }

            #[inline(always)]
            #[allow(irrefutable_let_patterns)]
            fn precede_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use $Alt::*;
                let (offset, states) = entry;

                resume_precede! {
                    states => { $(
                        $LabN: $VarN(_) => [{
                            *states = $VarN(self.$IdxN.init());
                        }] {
                            let $VarN(state) = states else { unreachable!() };
                            match self.$IdxN.precede(slice.split_at(*offset).1, state, eof) {
                                Ok(len) => *offset += len,
                                Err(e) => return e.raise_backtrack(*offset),
                            }
                        }
                    )+ }
                }

                Ok(*offset)
            }

            #[inline(always)]
            fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                slice.split_at(entry.0).0
            }
        }
    } };
}

macro_rules! impl_compoundable_for_tuples {
    (      $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_compoundable_for_tuples! { @
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_compoundable_for_tuple!( [<Alt $Lens1K>], $($LabN ~ [<P $OrdN>] ~ [<Var $OrdN>] ~ $IdxN)+ );

        impl_compoundable_for_tuples! { @
            $($Lens1N ~ $LabN ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_compoundable_for_tuples! {
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
        assert_eq!(
            __pat::<_, _, ParseError>(com((is_bin.., is_oct.., is_hex..)))
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            "0123456789abcdefABCDEF"
        )
    }
}
