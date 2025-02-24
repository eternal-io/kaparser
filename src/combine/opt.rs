use super::*;

#[inline(always)]
pub const fn opt<U, P>(opt: P) -> Optional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    Optional {
        opt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Optional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    opt: P,
    phantom: PhantomData<U>,
}

impl<U, P> Pattern2<U> for Optional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Some(self.opt.init2())
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        let (t, len) = self.opt.precede2(slice, entry.as_mut().unwrap(), eof)?;
        match t {
            Transfer::Rejected => {
                drop(entry.take());
                Some((Transfer::Accepted, 0))
            }
            t => Some((t, len)),
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        entry.map(|state| self.opt.extract2(slice, state))
    }
}
