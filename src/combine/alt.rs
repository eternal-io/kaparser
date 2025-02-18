use super::*;

#[inline(always)]
pub const fn alt<'i, U, A>(alt: A) -> Alternate<'i, U, A>
where
    U: 'i + ?Sized + Slice,
    A: Alternatable<'i, U>,
{
    Alternate {
        alt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Alternate<'i, U, A>
where
    U: 'i + ?Sized + Slice,
    A: Alternatable<'i, U>,
{
    alt: A,
    phantom: PhantomData<&'i U>,
}

pub trait Alternatable<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_alt(&self) -> Self::Internal;

    fn precede_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)>;

    fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, A> Pattern<'i, U> for Alternate<'i, U, A>
where
    U: 'i + ?Sized + Slice,
    A: Alternatable<'i, U>,
{
    type Captured = A::Captured;
    type Internal = A::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.alt.init_alt()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        self.alt.precede_alt(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.alt.extract_alt(slice, entry)
    }
}

macro_rules! impl_alternatable_for_tuple {
    ( $Alt:ident, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { $crate::common::paste! {
        impl<'i, U: 'i + ?Sized + Slice, $($GenN: Pattern<'i, U>),+> Alternatable<'i, U> for ($($GenN,)+) {
            type Captured = $Alt<$($GenN::Captured),+>;
            type Internal = $Alt<$($GenN::Internal),+>;

            #[inline(always)]
            fn init_alt(&self) -> Self::Internal {
                $Alt::Var1(self.0.init())
            }

            #[inline(always)]
            #[allow(irrefutable_let_patterns)]
            fn precede_alt(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
                use $Alt::*;

                resume_precede! {
                    entry => { $(
                        $LabN: $VarN(_) => [{
                            *entry = $VarN(self.$IdxN.init());
                        }] {
                            let $VarN(state) = entry else { unreachable!() };
                            let (t, len) = self.$IdxN.precede(slice, state, eof)?;
                            match t {
                                Transfer::Rejected => (),
                                t => return Some((t, len)),
                            }
                        }
                    )+ }
                }

                Some((Transfer::Rejected, 0))
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
    ) => { $crate::common::paste! {
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
