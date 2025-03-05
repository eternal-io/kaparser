use super::*;

#[inline(always)]
pub const fn opt<'i, U, E, P>(opt: P) -> Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Optional {
        opt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    opt: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.opt.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let res = self.opt.precede(slice, entry.as_mut().unwrap(), eof);
        if let Err(ref e) = res {
            if !e.is_unfulfilled() {
                *entry = None;
                return Ok(0);
            }
        }
        res
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        entry.map(|state| self.opt.extract(slice, state))
    }
}
