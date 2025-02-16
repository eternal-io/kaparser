use super::*;

#[inline(always)]
pub const fn opt<'i, U, P>(opt: P) -> Optional<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    Optional {
        opt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Optional<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    opt: P,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P> Pattern<'i, U> for Optional<'i, U, P>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.opt.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (t, len) = self.opt.precede(slice, entry.as_mut().unwrap(), eof)?;
        match t {
            Transfer::Rejected => {
                drop(entry.take());
                Ok((Transfer::Accepted, 0))
            }
            t => Ok((t, len)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        entry.map(|state| self.opt.extract(slice, state))
    }
}
