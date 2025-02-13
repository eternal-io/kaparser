use super::*;

pub const fn alt<'i, U: ?Sized + Slice, A: Alternatable<'i, U>>(alt: A) -> Alternate<'i, U, A> {
    Alternate {
        alt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Alternate<'i, U: ?Sized + Slice, A: Alternatable<'i, U>> {
    alt: A,
    phantom: PhantomData<&'i U>,
}

pub trait Alternatable<'i, U: ?Sized + Slice> {
    type Captured;
    type Internal: Clone;

    fn init_alt(&self) -> Self::Internal;

    fn proceed_alt(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult;

    fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U: ?Sized + Slice, A: Alternatable<'i, U>> Proceed<'i, U> for Alternate<'i, U, A> {
    type Captured = A::Captured;
    type Internal = A::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.alt.init_alt()
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        self.alt.proceed_alt(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.alt.extract_alt(slice, entry)
    }
}

macro_rules! impl_alternatable_for_tuple {
    ( $Len:literal, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { $crate::common::paste! {
        impl<'i, U: ?Sized + Slice, $($GenN: Proceed<'i, U>),+> Alternatable<'i, U> for ($($GenN,)+) {
            type Captured = [<Alt $Len>]<$($GenN::Captured),+>;
            type Internal = [<Alt $Len>]<$(Option<$GenN::Internal>),+>;

            #[inline(always)]
            fn init_alt(&self) -> Self::Internal {
                [<Alt $Len>]::[<Var $Len>](None)
            }

            #[inline(always)]
            #[allow(unreachable_patterns, irrefutable_let_patterns)]
            fn proceed_alt(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
                use [<Alt $Len>]::*;

                resume_proceed! {
                    'south: entry => { $(
                        $LabN: $VarN(_) => {
                            let state = match entry {
                                $VarN(some) => {
                                    cold_path();
                                    some
                                }
                                _ => {
                                    *entry = $VarN(None);
                                    let $VarN(none) = entry else { unreachable!() };
                                    none
                                }
                            }.get_or_insert_with(|| self.$IdxN.init());

                            match self.$IdxN.proceed(slice, state, eof)? {
                                Transfer::Rejected => (),

                                Transfer::Accepted(len) => {
                                    return Ok(Transfer::Accepted(len));
                                }
                                Transfer::Halt(len) => {
                                    cold_path();
                                    return Ok(Transfer::Halt(len));
                                }
                            }
                        }
                    )+ }
                }

                Ok(Transfer::Rejected)
            }

            #[inline(always)]
            fn extract_alt(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use [<Alt $Len>]::*;
                match entry { $(
                    $VarN(state) => $VarN(self.$IdxN.extract(slice, state.unwrap())),
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
        impl_alternatable_for_tuple!( $Lens1K, $($LabN ~ [<P $OrdN>] ~ [<Var $OrdN>] ~ $OrdN ~ $IdxN)+ );

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
