use super::*;

macro_rules! impl_pattern_for_tuple {
    ( $Len:literal, $( $LabN:lifetime ~ $GenN:ident ~ $ValN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Pattern<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
        {
            type Captured = ($($GenN::Captured,)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline(always)]
            fn init(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline(always)]
            fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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

                            match self.$IdxN.precede(slice.split_at(*off).1, state, eof) {
                                Ok(len) => offset = *off + len,
                                Err(e) => return e.raise_backtrack(*off),
                            }
                        }
                    )+ }
                }

                Ok(offset)
            }

            #[inline(always)]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1);
                )+
                ($($ValN,)+)
            }
        }
    } };
}

macro_rules! impl_pattern_for_tuples {
    (      $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_pattern_for_tuples! { @
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_pattern_for_tuple!( $Lens1K, $($LabN ~ [<P $OrdN>] ~ [<val $OrdN>] ~ $OrdN ~ $IdxN)+ );

        impl_pattern_for_tuples! { @
            $($Lens1N ~ $LabN ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_pattern_for_tuples! {
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
            opaque::<_, _, SimpleError>((is_bin.., is_oct.., is_hex..))
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ("01", "234567", "89abcdefABCDEF")
        )
    }
}
