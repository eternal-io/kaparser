use super::*;

#[inline(always)]
pub const fn wrap<'i, U, E, P>(body: P) -> Wrapper<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Wrapper {
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Wrapper<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Wrapper<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = (&'i U, P::Captured);
    type Internal = (usize, P::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.body.init())
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (offset, state) = entry;
        *offset = self.body.advance(slice, state, eof)?;
        Ok(*offset)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (offset, state) = entry;
        (slice.split_at(offset).0, self.body.extract(slice, state))
    }
}
