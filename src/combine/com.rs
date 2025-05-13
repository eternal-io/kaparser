use super::*;

#[doc(inline)]
pub use crate::com;

#[inline(always)]
pub const fn compound<'i, U, E, C>(com: C) -> Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    Compound {
        com,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    com: C,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Compoundable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_com(&self) -> Self::Internal;

    fn advance_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, C> Pattern<'i, U, E> for Compound<'i, U, E, C>
where
    U: ?Sized + Slice,
    E: Situation,
    C: Compoundable<'i, U, E>,
{
    type Captured = C::Captured;
    type Internal = C::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.com.init_com()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.com.advance_com(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.com.extract_com(slice, entry)
    }
}

macro_rules! impl_compoundable_for_tuple {
    ( $( $GenN:ident ~ $IdxN:tt )+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Compoundable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = &'i U;
            type Internal = usize;

            #[inline(always)]
            fn init_com(&self) -> Self::Internal {
                0
            }

            #[inline(always)]
            fn advance_com(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                *entry = 0;
            $( {
                let mut state = self.$IdxN.init();
                match self.$IdxN.advance(slice.split_at(*entry).1, &mut state, eof) {
                    Ok(len) => *entry += len,
                    Err(e) => return e.raise_backtrack(*entry),
                }
            } )+
                Ok(*entry)
            }

            #[inline(always)]
            fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                slice.split_at(entry).0
            }
        }
    } };
}

macro_rules! impl_compoundable_for_tuples {
    (      $OrdK:literal ~ $IdxK:tt
        $( $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_compoundable_for_tuples! { @
              $OrdK ~ $IdxK ;
            $($OrdM ~ $IdxM)*
        }
    };

    ( @ $( $OrdN:literal ~ $IdxN:tt )+ ;
           $OrdK:literal ~ $IdxK:tt
        $( $OrdM:literal ~ $IdxM:tt )*
    ) => { paste::paste! {
        impl_compoundable_for_tuple!( $([<P $OrdN>] ~ $IdxN)+ );

        impl_compoundable_for_tuples! { @
            $($OrdN ~ $IdxN)+
              $OrdK ~ $IdxK ;
            $($OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_compoundable_for_tuples! {
    1  ~ 0
    2  ~ 1
    3  ~ 2
    4  ~ 3
    5  ~ 4
    6  ~ 5
    7  ~ 6
    8  ~ 7
    9  ~ 8
    10 ~ 9
    11 ~ 10
    12 ~ 11
    13 ~ 12
    14 ~ 13
    15 ~ 14
    16 ~ 15
    17 ~ 16
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn com() {
        assert_eq!(
            simple_opaque(com!(is_bin.., is_oct.., is_hex..))
                .full_match("0123456789abcdefABCDEF")
                .unwrap(),
            "0123456789abcdefABCDEF"
        );
    }
}
