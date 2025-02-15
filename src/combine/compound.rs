use super::*;

#[inline(always)]
pub const fn com<'i, U, C>(com: C) -> Compound<'i, U, C>
where
    U: 'i + ?Sized + Slice,
    C: Compoundable<'i, U>,
{
    Compound {
        com,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Compound<'i, U, C>
where
    U: 'i + ?Sized + Slice,
    C: Compoundable<'i, U>,
{
    com: C,
    phantom: PhantomData<&'i U>,
}

pub trait Compoundable<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_com(&self) -> Self::Internal;

    fn proceed_com(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult;

    fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, C> Proceed<'i, U> for Compound<'i, U, C>
where
    U: 'i + ?Sized + Slice,
    C: Compoundable<'i, U>,
{
    type Captured = C::Captured;
    type Internal = C::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.com.init_com()
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        self.com.proceed_com(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.com.extract_com(slice, entry)
    }
}

macro_rules! impl_compoundable_for_tuple {
    ( $Alt:ident, $( $LabN:lifetime ~ $GenN:ident ~ $VarN:ident ~ $IdxN:tt )+ ) => { $crate::common::paste! {
        impl<'i, U: 'i + ?Sized + Slice, $($GenN: Proceed<'i, U>),+> Compoundable<'i, U> for ($($GenN,)+) {
            type Captured = &'i U;
            type Internal = (usize, $Alt<$($GenN::Internal),+>);

            #[inline(always)]
            fn init_com(&self) -> Self::Internal {
                (0, $Alt::Var1(self.0.init()))
            }

            #[inline(always)]
            #[allow(irrefutable_let_patterns)]
            fn proceed_com(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
                use $Alt::*;
                let (offset, states) = entry;

                proceed! {
                    states => { $(
                        $LabN: $VarN(_) => [{
                            *states = $VarN(self.$IdxN.init());
                        }] {
                            let $VarN(state) = states else { unreachable!() };
                            let (t, len) = self.$IdxN.proceed(slice.split_at(*offset).1, state, eof)?;
                            *offset += len;
                            match t {
                                Transfer::Accepted => (),
                                t => return Ok((t, *offset)),
                            }
                        }
                    )+ }
                }

                Ok((Transfer::Accepted, *offset))
            }

            #[inline(always)]
            fn extract_com(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                slice.split_at(entry.0).0
            }
        }
    } };
}

macro_rules! impl_compoundable_for_tuples {
    (      $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => {
        impl_compoundable_for_tuples! { @
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ;
           $Lens1K:literal ~ $LabK:lifetime ~ $OrdK:literal ~ $IdxK:tt
        $( $Lens1M:literal ~ $LabM:lifetime ~ $OrdM:literal ~ $IdxM:tt )*
    ) => { $crate::common::paste! {
        impl_compoundable_for_tuple!( [<Alt $Lens1K>], $($LabN ~ [<P $OrdN>] ~ [<Var $OrdN>] ~ $IdxN)+ );

        impl_compoundable_for_tuples! { @
            $($Lens1N ~ $LabN ~ $OrdN ~ $IdxN)+
              $Lens1K ~ $LabK ~ $OrdK ~ $IdxK ;
            $($Lens1M ~ $LabM ~ $OrdM ~ $IdxM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $LabN:lifetime ~ $OrdN:literal ~ $IdxN:tt )+ ; ) => {};
}

impl_compoundable_for_tuples! {
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
