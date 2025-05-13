use super::*;

#[doc(inline)]
pub use crate::ixs;

#[inline(always)]
pub const fn indexed_seq<'i, U, E, S>(ixs: S) -> IndexedSeq<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Indexedable<'i, U, E>,
{
    IndexedSeq {
        ixs,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct IndexedSeq<'i, U, E, S>
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

impl<'i, U, E, S> Pattern<'i, U, E> for IndexedSeq<'i, U, E, S>
where
    U: ?Sized + Slice,
    E: Situation,
    S: Indexedable<'i, U, E>,
{
    type Captured = S::Captured;
    type Internal = S::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.ixs.init_ixs()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.ixs.advance_ixs(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.ixs.extract_ixs(slice, entry)
    }
}

macro_rules! impl_indexedable_for_tuple {
    ( $Len:literal, $( $GenN:ident ~ $ValN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Indexedable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice,
            E: Situation,
        {
            type Captured = ($((usize, $GenN::Captured),)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline(always)]
            fn init_ixs(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline(always)]
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

            #[inline(always)]
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

macro_rules! impl_indexedable_for_tuples {
    (      $Lens1K:literal ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_indexedable_for_tuples! { @
              $Lens1K ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_indexedable_for_tuple!( $Lens1K, $([<P $OrdN>] ~ [<val $OrdN>] ~ $OrdN ~ $IdxN)+ );

        impl_indexedable_for_tuples! { @
            $($Lens1N ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_indexedable_for_tuples! {
    0  ~ 1  ~ 0
    1  ~ 2  ~ 1
    2  ~ 3  ~ 2
    3  ~ 4  ~ 3
    4  ~ 5  ~ 4
    5  ~ 6  ~ 5
    6  ~ 7  ~ 6
    7  ~ 8  ~ 7
    8  ~ 9  ~ 8
    9  ~ 10 ~ 9
    10 ~ 11 ~ 10
    11 ~ 12 ~ 11
    12 ~ 13 ~ 12
    13 ~ 14 ~ 13
    14 ~ 15 ~ 14
    15 ~ 16 ~ 15
    16 ~ 17 ~ 16
}

//------------------------------------------------------------------------------

macro_rules! impl_pattern_for_tuple {
    ( $Len:literal, $( $GenN:ident ~ $ValN:ident ~ $OrdN:literal ~ $IdxN:tt )+ ) => { paste::paste! {
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

                        match self.$IdxN.advance(slice.split_at(*off).1, state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

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
    (      $Lens1K:literal ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_pattern_for_tuples! { @
              $Lens1K ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_pattern_for_tuple!( $Lens1K, $([<P $OrdN>] ~ [<val $OrdN>] ~ $OrdN ~ $IdxN)+ );

        impl_pattern_for_tuples! { @
            $($Lens1N ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_pattern_for_tuples! {
    0  ~ 1  ~ 0
    1  ~ 2  ~ 1
    2  ~ 3  ~ 2
    3  ~ 4  ~ 3
    4  ~ 5  ~ 4
    5  ~ 6  ~ 5
    6  ~ 7  ~ 6
    7  ~ 8  ~ 7
    8  ~ 9  ~ 8
    9  ~ 10 ~ 9
    10 ~ 11 ~ 10
    11 ~ 12 ~ 11
    12 ~ 13 ~ 12
    13 ~ 14 ~ 13
    14 ~ 15 ~ 14
    15 ~ 16 ~ 15
    16 ~ 17 ~ 16
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn seq() {
        assert_eq!(
            simple_opaque((is_bin.., is_oct.., is_hex..))
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ("01", "234567", "89abcdefABCDEF")
        );
    }

    #[test]
    fn ixs() {
        assert_eq!(
            simple_opaque(ixs!(is_bin.., is_oct.., is_hex..))
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            ((0, "01"), (2, "234567"), (8, "89abcdefABCDEF"))
        );
    }
}
