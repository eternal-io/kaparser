use super::*;

#[inline]
pub const fn peek<'i, U, E, P>(body: P) -> Peek<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Peek {
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Peek<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Peek<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof).map(|_| 0)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}
