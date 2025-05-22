use super::*;
use core::ops::Range;

pub const fn seq<'i, U, E, S>(seq: S) -> Sequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Sequencable<'i, U, E>,
{
    Sequence {
        seq,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn ixs<'i, U, E, S>(ixs: S) -> IndexedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Indexedable<'i, U, E>,
{
    IndexedSequence {
        ixs,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn sps<'i, U, E, S>(sps: S) -> SpannedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Spannedable<'i, U, E>,
{
    SpannedSequence {
        sps,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Sequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Sequencable<'i, U, E>,
{
    seq: S,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Sequencable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_seq(&self) -> Self::Internal;

    fn advance_seq(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_seq(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, S> Pattern<'i, U, E> for Sequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Sequencable<'i, U, E>,
{
    type Captured = S::Captured;
    type Internal = S::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.seq.init_seq()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.seq.advance_seq(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.seq.extract_seq(slice, entry)
    }
}

macro_rules! impl_sequencable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN),+> Sequencable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = ($($GenN::Captured,)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline]
            fn init_seq(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance_seq(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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

                        match self.$IdxN.advance(slice.split_at(*off).1, state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract_seq(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1);
                )+
                ($($ValN,)+)
            }
        }
    } };
}

__generate_codes! { impl_sequencable_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

pub struct IndexedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Indexedable<'i, U, E>,
{
    ixs: S,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Indexedable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_ixs(&self) -> Self::Internal;

    fn advance_ixs(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_ixs(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, S> Pattern<'i, U, E> for IndexedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Indexedable<'i, U, E>,
{
    type Captured = S::Captured;
    type Internal = S::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.ixs.init_ixs()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.ixs.advance_ixs(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.ixs.extract_ixs(slice, entry)
    }
}

macro_rules! impl_indexedable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Indexedable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
        {
            type Captured = ($((usize, $GenN::Captured),)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline]
            fn init_ixs(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance_ixs(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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

                        match self.$IdxN.advance(slice.split_at(*off).1, state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract_ixs(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = ($ValN.0, self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1));
                )+
                ($($ValN,)+)
            }
        }
    } };
}

__generate_codes! { impl_indexedable_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

pub struct SpannedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Spannedable<'i, U, E>,
{
    sps: S,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Spannedable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_sps(&self) -> Self::Internal;

    fn advance_sps(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_sps(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, S> Pattern<'i, U, E> for SpannedSequence<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Spannedable<'i, U, E>,
{
    type Captured = S::Captured;
    type Internal = S::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.sps.init_sps()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.sps.advance_sps(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.sps.extract_sps(slice, entry)
    }
}

macro_rules! impl_spannedable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Spannedable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
        {
            type Captured = ($((Range<usize>, $GenN::Captured),)+);
            type Internal = ([<Check $Len>], ($((Range<usize>, $GenN::Internal),)+));

            #[inline]
            fn init_sps(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0..0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance_sps(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (span, state) = &mut states.$IdxN;
                        if likely(span.start == 0) {
                            span.start = offset;
                        }

                        match self.$IdxN.advance(slice.split_at(span.start).1, state, eof) {
                            Ok(len) => { offset = span.start + len ; span.end = offset }
                            Err(e) => return e.raise_backtrack(span.start),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract_sps(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = ($ValN.0.clone(), self.$IdxN.extract(slice.split_at($ValN.0.start).1, $ValN.1));
                )+
                ($($ValN,)+)
            }
        }
    } };
}

__generate_codes! { impl_spannedable_for_tuple ( P ~ val ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_seq() {
        assert_eq!(
            seq((is_bin.., is_oct.., is_hex..))
                .opaque_simple()
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ("01", "234567", "89abcdefABCDEF")
        );
    }

    #[test]
    fn test_ixs() {
        assert_eq!(
            ixs((is_bin.., is_oct.., is_hex..))
                .opaque_simple()
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ((0, "01"), (2, "234567"), (8, "89abcdefABCDEF"))
        );
    }

    #[test]
    fn test_sps() {
        assert_eq!(
            sps((is_bin.., is_oct.., is_hex..))
                .opaque_simple()
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ((0..2, "01"), (2..8, "234567"), (8..22, "89abcdefABCDEF"))
        );
    }
}
