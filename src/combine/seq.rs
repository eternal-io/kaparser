use super::*;

macro_rules! impl_sequencable_for_tuple {
    ( $Len:literal, $( $LabN:lifetime ~ $GenN:ident ~ $ValN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { $crate::common::paste! {
        impl<U: Slice2, $($GenN: Pattern2<U>),+> Pattern2<U> for ($($GenN,)+) {
            type Captured = ($($GenN::Captured,)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline(always)]
            fn init2(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init2()),)+))
            }

            #[inline(always)]
            fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
                use [<Check $Len>]::*;
                let (checkpoint, states) = entry;
                let mut offset = 0usize;

                resume_precede! {
                    *checkpoint => { $(
                        $LabN: [<Point $OrdN>] => [{
                            *checkpoint = [<Point $OrdN>];
                        }] {
                            let (off, state) = &mut states.$IdxN;
                            if likely(*off == 0) {
                                *off = offset;
                            }

                            let (t, len) = self.$IdxN.precede2(slice.split_at(*off).1, state, eof)?;
                            offset = *off + len;
                            match t {
                                Transfer::Accepted => (),
                                t => return Some((t, offset)),
                            }
                        }
                    )+ }
                }

                Some((Transfer::Accepted, offset))
            }

            #[inline(always)]
            fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = self.$IdxN.extract2(slice.split_at($ValN.0).1, $ValN.1);
                )+
                ($($ValN,)+)
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

// impl_sequencable_for_tuples! {
//     0  ~ 'p1  ~ 1  ~ 0
//     1  ~ 'p2  ~ 2  ~ 1
//     2  ~ 'p3  ~ 3  ~ 2
//     3  ~ 'p4  ~ 4  ~ 3
//     4  ~ 'p5  ~ 5  ~ 4
//     5  ~ 'p6  ~ 6  ~ 5
//     6  ~ 'p7  ~ 7  ~ 6
//     7  ~ 'p8  ~ 8  ~ 7
//     8  ~ 'p9  ~ 9  ~ 8
//     9  ~ 'p10 ~ 10 ~ 9
//     10 ~ 'p11 ~ 11 ~ 10
//     11 ~ 'p12 ~ 12 ~ 11
//     12 ~ 'p13 ~ 13 ~ 12
//     13 ~ 'p14 ~ 14 ~ 13
//     14 ~ 'p15 ~ 15 ~ 14
//     15 ~ 'p16 ~ 16 ~ 15
//     16 ~ 'p17 ~ 17 ~ 16
// }
