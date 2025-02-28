use super::*;

#[inline(always)]
#[doc(alias = "cond")]
pub const fn switch<U, P>(b: bool, body: P) -> Conditionate<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    Conditionate {
        cond: b,
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn filter<U, P, F>(f: F, body: P) -> Filter<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> bool,
{
    Filter {
        body,
        verify: f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Conditionate<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    cond: bool,
    body: P,
    phantom: PhantomData<U>,
}

impl<U, P> Pattern2<U> for Conditionate<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    type Captured = Option<P::Captured>;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match self.cond {
            true => self.body.precede2::<E>(slice, entry, eof),
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

pub struct Filter<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> bool,
{
    body: P,
    verify: F,
    phantom: PhantomData<U>,
}

impl<U, P, F> Pattern2<U> for Filter<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> bool,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
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
