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
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let res = self.opt.precede2::<E>(slice, entry.as_mut().unwrap(), eof);
        if let Err(ref e) = res {
            if !e.is_unfulfilled() {
                *entry = None;
                return Ok(0);
            }
        }
        res
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        entry.map(|state| self.opt.extract2(slice, state))
    }
}
