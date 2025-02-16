use super::*;

#[inline(always)]
pub const fn cut<'i, U, P>(body: P) -> Cut<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    Cut {
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Cut<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    body: P,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P> Pattern<'i, U> for Cut<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (t, len) = self.body.precede(slice, entry, eof)?;
        Ok((t.cut(), len))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}
