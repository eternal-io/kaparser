use super::*;

pub fn alt<'i, U: ?Sized + Slice, A: Alternatable<'i, U>>(alt: A) -> Alternate<'i, U, A> {
    Alternate {
        alt,
        phantom: PhantomData,
    }
}

pub struct Alternate<'i, U: ?Sized + Slice, A: Alternatable<'i, U>> {
    pub(super) alt: A,
    phantom: PhantomData<&'i U>,
}

pub trait Alternatable<'i, U: ?Sized + Slice> {
    type Capture;
    type State: Default;

    fn proceed_alt(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult;

    fn extract_alt(&self, slice: &'i U, entry: Self::State) -> Self::Capture;
}

macro_rules! impl_alternatable_for_tuple {
    ( $Sum:ident, $( $GenN:ident ~ $VarN:ident ~ $IdxN:tt )+ ) => {
        impl<'i, U: ?Sized, $($GenN: Proceed<'i, U>),+> Alternatable<'i, U> for ($($GenN,)+) {
            type Capture = $Sum<$($GenN::Captured),+>;;
            type State = (u8, $Sum<$($GenN::Checkpoint),+>>);

            #[inline(always)]
            fn proceed_alt(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult {
                let (checkpoint, children) = entry;
                resume_proceed! {
                    checkpoint { $(
                        $IdxN {
                            let (off, state) = &mut children.$ValN;
                            if *off == 0 {
                                *off = tot_len;
                            }

                            match self.$IdxN.proceed(slice.split_at(*off).1, state, eof)? {
                                Transfer::Rejected  => return Ok(Transfer::Rejected),
                                Transfer::Halt(len) => return Ok(Transfer::Halt(*off + len)),
                                Transfer::Accepted(len) => {
                                    *off += len;
                                    tot_len = *off;
                                }
                            }
                        }
                    )+ }
                }
            }

            fn extract_alt(&self, slice: &'i U, entry: Self::State) -> Self::Capture;
        }
    };
}

macro_rules! impl_alternatable_for_tuples {
    (      $SumK:ident ~ $GenK:ident ~ $VarK:ident ~ $IdxK:tt
        $( $SumM:ident ~ $GenM:ident ~ $VarM:ident ~ $IdxM:tt )*
    ) => {
        impl_alternatable_for_tuples! { @
              $SumK ~ $GenK ~ $VarK ~ $IdxK ;
            $($SumM ~ $GenM ~ $VarM ~ $IdxM)*
        }
    };

    ( @ $( $SumN:ident ~ $GenN:ident ~ $VarN:ident ~ $IdxN:tt )+ ;
           $SumK:ident ~ $GenK:ident ~ $VarK:ident ~ $IdxK:tt
        $( $SumM:ident ~ $GenM:ident ~ $VarM:ident ~ $IdxM:tt )*
    ) => { $crate::common::paste! {
        impl_alternatable_for_tuple!( $SumK, $($GenN ~ $VarN ~ $IdxN)+ );

        impl_alternatable_for_tuples! { @
            $($SumN ~ $GenN ~ $VarN ~ $IdxN)+
              $SumK ~ $GenK ~ $VarK ~ $IdxK ;
            $($SumM ~ $GenM ~ $VarM ~ $IdxM)*
        }
    } };

    ( @ $( $SumN:ident ~ $GenN:ident ~ $VarN:ident ~ $IdxN:tt )+ ; ) => {};
}

// impl_alternatable_for_tuples! {
//     Sum0  ~ P1  ~ Var1  ~ 0
//     Sum1  ~ P2  ~ Var2  ~ 1
//     Sum2  ~ P3  ~ Var3  ~ 2
//     Sum3  ~ P4  ~ Var4  ~ 3
//     Sum4  ~ P5  ~ Var5  ~ 4
//     Sum5  ~ P6  ~ Var6  ~ 5
//     Sum6  ~ P7  ~ Var7  ~ 6
//     Sum7  ~ P8  ~ Var8  ~ 7
//     Sum8  ~ P9  ~ Var9  ~ 8
//     Sum9  ~ P10 ~ Var10 ~ 9
//     Sum10 ~ P11 ~ Var11 ~ 10
//     Sum11 ~ P12 ~ Var12 ~ 11
//     Sum12 ~ P13 ~ Var13 ~ 12
//     Sum13 ~ P14 ~ Var14 ~ 13
//     Sum14 ~ P15 ~ Var15 ~ 14
//     Sum15 ~ P16 ~ Var16 ~ 15
//     Sum16 ~ P17 ~ Var17 ~ 16
// }
