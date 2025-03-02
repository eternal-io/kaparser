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

#[inline(always)]
pub const fn unwrap<U, P>(body: P) -> Unwrap<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    Unwrap {
        body,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn unwrap_or<U, P, C>(default: C, body: P) -> UnwrapOr<U, P, C>
where
    U: Slice2,
    P: Pattern2<U, Captured = C>,
    C: Clone,
{
    UnwrapOr {
        body,
        default,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn unwrap_or_else<U, P, F>(f: F, body: P) -> UnwrapOrElse<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn() -> P::Captured,
{
    UnwrapOrElse {
        body,
        f,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn unwrap_or_default<U, P, D>(body: P) -> UnwrapOrDefault<U, P, D>
where
    U: Slice2,
    P: Pattern2<U, Captured = D>,
    D: Default,
{
    UnwrapOrDefault {
        body,
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

//------------------------------------------------------------------------------

pub struct Unwrap<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    body: P,
    phantom: PhantomData<U>,
}

impl<U, P> Pattern2<U> for Unwrap<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        Ok(self.body.precede2::<E>(slice, entry, eof).expect("unexpected input"))
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract2(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct UnwrapOr<U, P, C>
where
    U: Slice2,
    P: Pattern2<U, Captured = C>,
    C: Clone,
{
    body: P,
    default: C,
    phantom: PhantomData<U>,
}

impl<U, P, C> Pattern2<U> for UnwrapOr<U, P, C>
where
    U: Slice2,
    P: Pattern2<U, Captured = C>,
    C: Clone,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Some(self.body.init2())
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match entry {
            None => Ok(0),
            Some(state) => {
                let res = self.body.precede2::<E>(slice, state, eof); // TODO: optimization?
                if let Err(ref e) = res {
                    if !e.is_unfulfilled() {
                        *entry = None;
                    }
                }
                res
            }
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => self.default.clone(),
            Some(state) => self.body.extract2(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct UnwrapOrElse<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn() -> P::Captured,
{
    body: P,
    f: F,
    phantom: PhantomData<U>,
}

impl<U, P, F> Pattern2<U> for UnwrapOrElse<U, P, F>
where
    U: Slice2,
    P: Pattern2<U>,
    F: Fn() -> P::Captured,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Some(self.body.init2())
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match entry {
            None => Ok(0),
            Some(state) => {
                let res = self.body.precede2::<E>(slice, state, eof);
                if let Err(ref e) = res {
                    if !e.is_unfulfilled() {
                        *entry = None;
                    }
                }
                res
            }
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => (self.f)(),
            Some(state) => self.body.extract2(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct UnwrapOrDefault<U, P, D>
where
    U: Slice2,
    P: Pattern2<U, Captured = D>,
    D: Default,
{
    body: P,
    phantom: PhantomData<U>,
}

impl<U, P, D> Pattern2<U> for UnwrapOrDefault<U, P, D>
where
    U: Slice2,
    P: Pattern2<U, Captured = D>,
    D: Default,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        Some(self.body.init2())
    }
    #[inline(always)]
    fn precede2<E: Situation>(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match entry {
            None => Ok(0),
            Some(state) => {
                let res = self.body.precede2::<E>(slice, state, eof);
                if let Err(ref e) = res {
                    if !e.is_unfulfilled() {
                        *entry = None;
                    }
                }
                res
            }
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => Default::default(),
            Some(state) => self.body.extract2(slice, state),
        }
    }
}
