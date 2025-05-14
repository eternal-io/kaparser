use super::*;
use core::mem;

#[doc(inline)]
pub use crate::disp;

#[inline(always)]
pub const fn dispatch<'i, U, E, D>(disp: D) -> Dispatch<'i, U, E, D>
where
    U: ?Sized + Slice,
    E: Situation,
    D: Dispatchable<'i, U, E>,
{
    Dispatch {
        disp,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Dispatch<'i, U, E, D>
where
    U: ?Sized + Slice,
    E: Situation,
    D: Dispatchable<'i, U, E>,
{
    disp: D,
    phantom: PhantomData<(&'i U, E)>,
}

pub trait Dispatchable<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_disp(&self) -> Self::Internal;

    fn advance_disp(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract_disp(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;
}

impl<'i, U, E, D> Pattern<'i, U, E> for Dispatch<'i, U, E, D>
where
    U: ?Sized + Slice,
    E: Situation,
    D: Dispatchable<'i, U, E>,
{
    type Captured = D::Captured;
    type Internal = D::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.disp.init_disp()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.disp.advance_disp(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.disp.extract_disp(slice, entry)
    }
}

macro_rules! impl_dispatchable_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $VarN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, P0, $($GenN),+> Dispatchable<'i, U, E> for (P0, ($((P0::Captured, $GenN),)+))
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
            P0: Pattern<'i, U, E>,
            P0::Captured: PartialEq,
          $($GenN: Pattern<'i, U, E>,)+
        {
            type Captured = [<Alt $Len>]<$($GenN::Captured),+>;
            type Internal = Alt3<P0::Internal, (), (usize, [<Alt $Len>]<$($GenN::Internal),+>)>;

            #[inline(always)]
            fn init_disp(&self) -> Self::Internal {
                Alt3::Var1(self.0.init())
            }

            #[inline(always)]
            #[allow(unsafe_code)]
            #[allow(irrefutable_let_patterns)]
            fn advance_disp(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Alt $Len>]::*;

                if let Alt3::Var1(state) = entry {
                    let offset = self.0.advance(slice, state, eof)?;
                    let Alt3::Var1(state) = mem::replace(entry, Alt3::Var2(())) else {
                        unreachable!()
                    };

                    // SAFETY: The captured is only used temporarily in this function.
                    let captured = self.0.extract(unsafe { mem::transmute::<&U, &'i U>(slice) }, state);
                    let internal = $( if captured == self.1.$IdxN.0 {
                        $VarN(self.1.$IdxN.1.init())
                    } else )+ {
                        return E::raise_reject_at(0);
                    };

                    *entry = Alt3::Var3((offset, internal));
                }

                let Alt3::Var3((offset, entry)) = entry else {
                    panic!("contract violation")
                };
                let slice = slice.split_at(*offset).1;

                match entry { $(
                    $VarN(state) => self.1.$IdxN.1.advance(slice, state, eof),
                )+ }.map(|off| *offset + off)
            }

            #[inline(always)]
            fn extract_disp(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use [<Alt $Len>]::*;

                let Alt3::Var3((_ffset, state)) = entry else {
                    panic!("contract violation")
                };

                match state { $(
                    $VarN(state) => $VarN(self.1.$IdxN.1.extract(slice, state)),
                )+ }
            }
        }
    } };
}

__generate_codes! { impl_dispatchable_for_tuple ( P ~ Var ) }

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn disp() {
        let pat = simple_opaque(
            disp! {
                take(2, ..);
                "0x" => def::hex_().filter_map(|s| u64::from_str_radix(s, 16).ok()),
                "0o" => def::oct_().filter_map(|s| u64::from_str_radix(s, 8).ok()),
                "0b" => def::bin_().filter_map(|s| u64::from_str_radix(s, 2).ok()),
            }
            .converge(),
        );

        assert_eq!(pat.full_match("0x89AB").unwrap(), 0x89AB);
        assert_eq!(pat.full_match("0o4567").unwrap(), 0o4567);
        assert_eq!(pat.full_match("0b1110").unwrap(), 0b1110);
    }
}
