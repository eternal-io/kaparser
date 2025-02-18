//! Infallible conversion.

use super::*;

#[inline(always)]
pub const fn map<'i, U, P, F, O>(f: F, body: P) -> Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> O,
{
    Map {
        body,
        then: f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> O,
{
    body: P,
    then: F,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, F, O> Pattern<'i, U> for Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> O,
{
    type Captured = O;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.then)(self.body.extract(slice, entry))
    }
}
