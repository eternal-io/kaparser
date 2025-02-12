use super::*;

pub fn seq<'i, U: ?Sized + Slice, S: Sequencable<'i, U>>(seq: S) -> Sequence<'i, U, S> {
    Sequence {
        seq,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Sequence<'i, U: ?Sized + Slice, S: Sequencable<'i, U>> {
    seq: S,
    phantom: PhantomData<&'i U>,
}

pub trait Sequencable<'i, U: ?Sized + Slice> {
    type Captured;
    type Internal: Clone;

    fn init_seq() -> Self::Internal;

    fn proceed_seq(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult;

    fn extract_seq(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U: ?Sized + Slice, S: Sequencable<'i, U>> Proceed<'i, U> for Sequence<'i, U, S> {
    type Captured = S::Captured;
    type Internal = S::Internal;

    #[inline(always)]
    fn init() -> Self::Internal {
        S::init_seq()
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        self.seq.proceed_seq(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.seq.extract_seq(slice, entry)
    }
}

macro_rules! impl_sequencable_for_tuple {
    ( $Len:literal, $( $LabN:lifetime ~ $GenN:ident ~ $ValN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { $crate::common::paste! {
        impl<'i, U: ?Sized + Slice, $($GenN: Proceed<'i, U>),+> Sequencable<'i, U> for ($($GenN,)+) {
            type Captured = ($($GenN::Captured,)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline(always)]
            fn init_seq() -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, $GenN::init()),)+))
            }

            #[inline(always)]
            fn proceed_seq(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
                use [<Check $Len>]::*;
                let (checkpoint, children) = entry;
                let mut tot_len = 0usize;

                resume_proceed! {
                    'south: *checkpoint => { $(
                        $LabN: [<Point $OrdN>] => {
                            *checkpoint = [<Point $OrdN>];

                            let (off, state) = &mut children.$IdxN;
                            if likely(*off == 0) {
                                *off = tot_len;
                            }

                            match self.$IdxN.proceed(slice.split_at(*off).1, state, eof)? {
                                Transfer::Accepted(len) => {
                                    *off += len;
                                    tot_len = *off;
                                }
                                Transfer::Rejected => {
                                    return Ok(Transfer::Rejected);
                                }
                                Transfer::Halt(len) => {
                                    cold_path();
                                    return Ok(Transfer::Halt(*off + len))
                                }
                            }
                        }
                    )+ }
                }

                Ok(Transfer::Accepted(tot_len))
            }

            #[inline(always)]
            #[allow(unsafe_code)]
            fn extract_seq(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                let ($($ValN,)+) = entry.1;
                let mut captuer = MaybeUninit::<Self::Captured>::uninit();
                $(
                    let cap = self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1);
                    unsafe {
                        (&raw mut (*captuer.as_mut_ptr()).$IdxN).write(cap);
                    }
                )+
                unsafe { captuer.assume_init() }
            }
        }
    } };
}

macro_rules! impl_sequencable_for_tuples {
    (      $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_sequencable_for_tuples! { @
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { $crate::common::paste! {
        impl_sequencable_for_tuple!( $Lens1K, $($LabN ~ [<P $OrdN>] ~ [<val $OrdN>] ~ $OrdN ~ $IdxN)+ );

        impl_sequencable_for_tuples! { @
            $($Lens1N ~ $LabN ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_sequencable_for_tuples! {
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
