use super::*;

#[inline(always)]
pub const fn cond<U, E, P>(b: bool, body: P) -> Conditionate<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    Conditionate {
        cond: b,
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
#[doc(alias = "filter")]
pub const fn verify<U, E, P, F>(f: F, body: P) -> Verify<U, E, P, F>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
    F: Fn(P::Captured) -> bool,
{
    Verify {
        body,
        verify: f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Conditionate<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    cond: bool,
    body: P,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern2<U, E> for Conditionate<U, E, P>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match self.cond {
            true => self.body.precede2(slice, entry, eof),
            false => Ok(0),
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match self.cond {
            true => Some(self.body.extract2(slice, entry)),
            false => None,
        }
    }
}

//------------------------------------------------------------------------------

pub struct Verify<U, E, P, F>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
    F: Fn(P::Captured) -> bool,
{
    body: P,
    verify: F,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P, F> Pattern2<U, E> for Verify<U, E, P, F>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
    F: Fn(P::Captured) -> bool,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let len = self.body.precede2(slice, entry, eof)?;
        ((self.verify)(self.body.extract2(slice, entry.clone())))
            .then_some(len)
            .ok_or(E::reject_at(len))
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract2(slice, entry)
    }
}
