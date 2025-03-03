use super::*;

#[inline(always)]
pub const fn alt<U, E, A>(alt: A) -> Alternate<U, E, A>
where
    U: Slice,
    E: Situation,
    A: Alternatable<U, E>,
{
    Alternate {
        alt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Alternate<U, E, A>
where
    U: Slice,
    E: Situation,
    A: Alternatable<U, E>,
{
    alt: A,
    phantom: PhantomData<(U, E)>,
}

pub trait Alternatable<U, E>
where
    U: Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_alt(&self) -> Self::Internal;

    fn precede_alt(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E>;

    fn extract_alt(&self, slice: U, entry: Self::Internal) -> Self::Captured;
}

impl<U, E, A> Pattern<U, E> for Alternate<U, E, A>
where
    U: Slice,
    E: Situation,
    A: Alternatable<U, E>,
{
    type Captured = A::Captured;
    type Internal = A::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.alt.init_alt()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.alt.precede_alt(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.alt.extract_alt(slice, entry)
    }
}

macro_rules! impl_alternatable_for_tuple {
    ( $Alt:ident, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<U, E, $($GenN: Pattern<U, E>),+> Alternatable<U, E> for ($($GenN,)+)
        where
            U: Slice,
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
            fn precede_alt(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
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
            fn extract_alt(&self, slice: U, entry: Self::Internal) -> Self::Captured {
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
