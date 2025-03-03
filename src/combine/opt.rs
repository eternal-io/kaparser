use super::*;

#[inline(always)]
pub const fn opt<U, E, P>(opt: P) -> Optional<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    Optional {
        opt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Optional<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    opt: P,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern2<U, E> for Optional<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Some(self.opt.init2())
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let res = self.opt.precede2(slice, entry.as_mut().unwrap(), eof);
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
