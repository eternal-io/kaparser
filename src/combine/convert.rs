use super::*;

#[inline(always)]
pub const fn map<U, P, F, O>(f: F, body: P) -> Map<U, P, F, O>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> O,
{
    Map {
        body,
        then: f,
        phantom: PhantomData,
    }
}

#[inline(always)]
#[doc(alias = "and_then")]
pub const fn complex<U, P, Q>(body: P, then: Q) -> Complex<U, P, Q>
where
    U: Slice2,
    P: Pattern2<U>,
    Q: Pattern2<U>,
{
    Complex {
        body,
        then,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Map<U, P, F, O>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> O,
{
    body: P,
    then: F,
    phantom: PhantomData<U>,
}

impl<U, P, F, O> Pattern2<U> for Map<U, P, F, O>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn(P::Captured) -> O,
{
    type Captured = O;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body.precede2(slice, entry, eof)
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        (self.then)(self.body.extract2(slice, entry))
    }
}

//------------------------------------------------------------------------------

pub struct Complex<U, P, Q>
where
    U: Slice2,
    P: Pattern2<U>,
    Q: Pattern2<U>,
{
    body: P,
    then: Q,
    phantom: PhantomData<U>,
}

impl<U, P, Q> Pattern2<U> for Complex<U, P, Q>
where
    U: Slice2,
    P: Pattern2<U>,
    Q: Pattern2<U>,
{
    type Captured = Q::Captured;
    type Internal = Alt2<P::Internal, Q::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Alt2::Var1(self.body.init2())
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let Alt2::Var1(state) = entry else {
            panic!("contract violation")
        };
        let len = self.body.precede2(slice, state, eof)?;

        *entry = Alt2::Var2(self.then.init2());

        let Alt2::Var2(state) = entry else { unreachable!() };
        self.then.precede2::<E>(slice.split_at(len).0, state, true)
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract2(slice, state)
    }
}
