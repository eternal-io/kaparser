use super::*;

#[inline(always)]
pub const fn with<'i, U, P, F, Ḟ>(then: F, body: P) -> With<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    F: Fn(P::Captured) -> Ḟ,
{
    With {
        body,
        then,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct With<'i, U, P, F, ඞ>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    F: Fn(P::Captured) -> ඞ,
{
    body: P,
    then: F,
    phantom: PhantomData<(&'i U, ඞ)>,
}

impl<'i, U, P, F, ඞ> Precede<'i, U> for With<'i, U, P, F, ඞ>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    F: Fn(P::Captured) -> ඞ,
{
    type Captured = ඞ;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.then)(self.body.extract(slice, entry))
    }
}
