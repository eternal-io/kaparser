use super::*;

#[inline(always)]
pub const fn cond<'i, U, E, P>(b: bool, body: P) -> Conditionate<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Conditionate {
        cond: b,
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Conditionate<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    cond: bool,
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Conditionate<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match self.cond {
            true => self.body.precede(slice, entry, eof),
            false => Ok(0),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match self.cond {
            true => Some(self.body.extract(slice, entry)),
            false => None,
        }
    }
}
