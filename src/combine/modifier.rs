use super::*;

#[inline(always)]
pub const fn parallel<'i, U, E, P>(body: P) -> Parallel<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Parallel {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn complex<'i, U, E, P, Q>(body: P, then: Q) -> Complex<'i, U, E, P, Q>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    Complex {
        body,
        then,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn desc<'i, U, E, P>(desc: E::Description, body: P) -> Describe<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<'i, U, E>,
{
    Describe {
        body,
        desc,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn desc_with<'i, U, E, P, F>(f: F, body: P) -> DescribeWith<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&E) -> E::Description,
{
    DescribeWith {
        body,
        f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Parallel<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Parallel<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = (P::Captured, &'i U);
    type Internal = (P::Internal, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (self.body.init(), 0)
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (state, offset) = entry;
        *offset = self.body.advance(slice, state, eof)?;
        Ok(*offset)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (state, offset) = entry;
        (self.body.extract(slice, state), slice.split_at(offset).0)
    }
}

//------------------------------------------------------------------------------

pub struct Complex<'i, U, E, P, Q>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    body: P,
    then: Q,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, Q> Pattern<'i, U, E> for Complex<'i, U, E, P, Q>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    type Captured = Q::Captured;
    type Internal = Alt2<P::Internal, Q::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Alt2::Var1(self.body.init())
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let Alt2::Var1(state) = entry else {
            panic!("contract violation")
        };
        let len = self.body.advance(slice, state, eof)?;

        *entry = Alt2::Var2(self.then.init());

        let Alt2::Var2(state) = entry else { unreachable!() };
        self.then.advance(slice.split_at(len).0, state, true)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract(slice, state)
    }
}

//------------------------------------------------------------------------------

pub struct Describe<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<'i, U, E>,
{
    body: P,
    desc: E::Description,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Describe<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body
            .advance(slice, entry, eof)
            .map_err(|e| e.describe(self.desc.clone()))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct DescribeWith<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&E) -> E::Description,
{
    body: P,
    f: F,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, F> Pattern<'i, U, E> for DescribeWith<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&E) -> E::Description,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof).map_err(|e| {
            let desc = (self.f)(&e);
            e.describe(desc)
        })
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}
