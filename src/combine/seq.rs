use super::*;

#[inline]
pub const fn ixs<S>(ixs: S) -> IndexedSeq<S> {
    IndexedSeq { ixs }
}

#[inline]
pub const fn sps<S>(sps: S) -> SpannedSeq<S> {
    SpannedSeq { sps }
}

//------------------------------------------------------------------------------

macro_rules! impl_sequence_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN),+> Pattern<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = ($($GenN::Captured,)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline]
            fn init(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (off, state) = &mut states.$IdxN;
                        if likely(*off == 0) {
                            *off = offset;
                        }

                        match self.$IdxN.advance(slice.after(*off), state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = self.$IdxN.extract(slice.after($ValN.0), $ValN.1);
                )+
                ($($ValN,)+)
            }
        }
    } };
}

__generate_codes! { impl_sequence_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

pub struct IndexedSeq<S> {
    ixs: S,
}

macro_rules! impl_indexed_seq_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        #[doc(hidden)]
        impl<'i, U, E, $($GenN),+> Pattern<'i, U, E> for IndexedSeq<($($GenN,)+)>
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = ($((usize, $GenN::Captured),)+);
            type Internal = (usize, [<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline]
            fn init(&self) -> Self::Internal {
                (0, [<Check $Len>]::Point1, ($((0, self.ixs.$IdxN.init()),)+))
            }

            #[inline]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (_base_off, checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (off, state) = &mut states.$IdxN;
                        if likely(*off == 0) {
                            *off = offset;
                        }

                        match self.ixs.$IdxN.advance(slice.after(*off), state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                let (base_off, _check, states) = entry;
                $(
                    let (off, entry) = states.$IdxN;
                    let $ValN = (base_off + off, self.ixs.$IdxN.extract(slice.after(off), entry));
                )+
                ($($ValN,)+)
            }

            #[inline]
            fn inject_base_off(&self, entry: &mut Self::Internal, base_off: usize) {
                entry.0 = base_off;
                $(
                    self.ixs.$IdxN.inject_base_off(&mut entry.2.$IdxN.1, base_off);
                )+
            }
        }
    } };
}

__generate_codes! { impl_indexed_seq_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

pub struct SpannedSeq<S> {
    sps: S,
}

macro_rules! impl_spanned_seq_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        #[doc(hidden)]
        impl<'i, U, E, $($GenN),+> Pattern<'i, U, E> for SpannedSeq<($($GenN,)+)>
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = ($((Range<usize>, $GenN::Captured),)+);
            type Internal = (usize, [<Check $Len>], ($((Range<usize>, $GenN::Internal),)+));

            #[inline]
            fn init(&self) -> Self::Internal {
                (0, [<Check $Len>]::Point1, ($((0..0, self.sps.$IdxN.init()),)+))
            }

            #[inline]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (_base_off, checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (span, state) = &mut states.$IdxN;
                        if likely(span.start == 0) {
                            span.start = offset;
                        }

                        match self.sps.$IdxN.advance(slice.after(span.start), state, eof) {
                            Ok(len) => { offset = span.start + len ; span.end = offset }
                            Err(e) => return e.raise_backtrack(span.start),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                let (base_off, _check, states) = entry;
                $(
                    let (span, entry) = states.$IdxN;
                    let $ValN = (
                        base_off + span.start..base_off + span.end,
                        self.sps.$IdxN.extract(slice.after(span.start), entry),
                    );
                )+
                ($($ValN,)+)
            }

            #[inline]
            fn inject_base_off(&self, entry: &mut Self::Internal, base_off: usize) {
                entry.0 = base_off;
                $(
                    self.sps.$IdxN.inject_base_off(&mut entry.2.$IdxN.1, base_off);
                )+
            }
        }
    } };
}

__generate_codes! { impl_spanned_seq_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;

//     #[test]
//     fn test_seq() {
//         assert_eq!(
//             opaque_simple((is_bin.., is_oct.., is_hex..))
//                 .fullmatch("0123456789abcdefABCDEF")
//                 .unwrap(),
//             ("01", "234567", "89abcdefABCDEF")
//         );
//     }

//     #[test]
//     fn test_ixs() {
//         assert_eq!(
//             opaque_simple(ixs((is_bin.., is_oct.., is_hex..)))
//                 .fullmatch("0123456789abcdefABCDEF")
//                 .unwrap(),
//             ((0, "01"), (2, "234567"), (8, "89abcdefABCDEF"))
//         );
//     }

//     #[test]
//     fn test_sps() {
//         assert_eq!(
//             opaque_simple(sps((is_bin.., is_oct.., is_hex..)))
//                 .fullmatch("0123456789abcdefABCDEF")
//                 .unwrap(),
//             ((0..2, "01"), (2..8, "234567"), (8..22, "89abcdefABCDEF"))
//         );
//     }
// }
