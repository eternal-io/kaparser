use super::*;
use core::mem::MaybeUninit;

pub fn seq<'i, U: ?Sized + Slice, S: Sequencable<'i, U>>(seq: S) -> Sequence<'i, U, S> {
    Sequence {
        seq,
        phantom: PhantomData,
    }
}

pub struct Sequence<'i, U: ?Sized + Slice, S: Sequencable<'i, U>> {
    pub(super) seq: S,
    phantom: PhantomData<&'i U>,
}

pub trait Sequencable<'i, U: ?Sized + Slice> {
    type Capture;
    type State: Default;

    fn proceed_seq(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult;

    fn extract_seq(&self, slice: &'i U, entry: Self::State) -> Self::Capture;
}

macro_rules! impl_sequencable_for_tuple {
    ( $Prod:ident, $( $GenN:ident ~ $ValN:ident ~ $IdxN:tt )+ ) => {
        impl<'i, U: ?Sized + Slice, $($GenN: Proceed<'i, U>),+> Sequencable<'i, U> for ($($GenN,)+) {
            type Capture = $Prod<$($GenN::Capture),+>;
            type State = (u8, $Prod<$((usize, $GenN::State)),+>);

            #[inline(always)]
            fn proceed_seq(&self, slice: &'i U, entry: &mut Self::State, eof: bool) -> ProceedResult {
                let (checkpoint, children) = entry;
                let mut tot_len = 0usize;
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
                Ok(Transfer::Accepted(tot_len))
            }

            #[inline(always)]
            fn extract_seq(&self, slice: &'i U, entry: Self::State) -> Self::Capture {
                let mut captuer = MaybeUninit::<Self::Capture>::uninit();
                let ptr = captuer.as_mut_ptr();
                let $Prod { $($ValN),+ } = entry.1;
                $(
                    let cap = self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1);
                    unsafe {
                        (&raw mut (*ptr).$ValN).write(cap);
                    }
                )+
                unsafe { captuer.assume_init() }
            }
        }
    };
}

macro_rules! impl_sequencable_for_tuples {
    (      $ProdK:ident ~ $GenK:ident ~ $ValK:ident ~ $IdxK:tt
        $( $ProdM:ident ~ $GenM:ident ~ $ValM:ident ~ $IdxM:tt )*
    ) => {
        impl_sequencable_for_tuples! { @
              $ProdK ~ $GenK ~ $ValK ~ $IdxK ;
            $($ProdM ~ $GenM ~ $ValM ~ $IdxM)*
        }
    };

    ( @ $( $ProdN:ident ~ $GenN:ident ~ $ValN:ident ~ $IdxN:tt )+ ;
           $ProdK:ident ~ $GenK:ident ~ $ValK:ident ~ $IdxK:tt
        $( $ProdM:ident ~ $GenM:ident ~ $ValM:ident ~ $IdxM:tt )*
    ) => { $crate::common::paste! {
        impl_sequencable_for_tuple!( $ProdK, $($GenN ~ $ValN ~ $IdxN)+ );

        impl_sequencable_for_tuples! { @
            $($ProdN ~ $GenN ~ $ValN ~ $IdxN)+
              $ProdK ~ $GenK ~ $ValK ~ $IdxK ;
            $($ProdM ~ $GenM ~ $ValM ~ $IdxM)*
        }
    } };

    ( @ $( $ProdN:ident ~ $GenN:ident ~ $ValN:ident ~ $IdxN:tt )+ ; ) => {};
}

// impl_sequencable_for_tuples! {
//     Product0  ~ P1  ~ val1  ~ 0
//     Product1  ~ P2  ~ val2  ~ 1
//     Product2  ~ P3  ~ val3  ~ 2
//     Product3  ~ P4  ~ val4  ~ 3
//     Product4  ~ P5  ~ val5  ~ 4
//     Product5  ~ P6  ~ val6  ~ 5
//     Product6  ~ P7  ~ val7  ~ 6
//     Product7  ~ P8  ~ val8  ~ 7
//     Product8  ~ P9  ~ val9  ~ 8
//     Product9  ~ P10 ~ val10 ~ 9
//     Product10 ~ P11 ~ val11 ~ 10
//     Product11 ~ P12 ~ val12 ~ 11
//     Product12 ~ P13 ~ val13 ~ 12
//     Product13 ~ P14 ~ val14 ~ 13
//     Product14 ~ P15 ~ val15 ~ 14
//     Product15 ~ P16 ~ val16 ~ 15
//     Product16 ~ P17 ~ val17 ~ 16
// }
