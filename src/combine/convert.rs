use super::*;

#[inline(always)]
pub const fn map<U, E, P, F, Out>(op: F, body: P) -> Map<U, E, P, F, Out>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(P::Captured) -> Out,
{
    Map {
        body,
        op,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn map_err<U, E1, P, F, E2>(op: F, body: P) -> MapErr<U, E1, P, F, E2>
where
    U: Slice,
    E1: Situation,
    P: Pattern<U, E1>,
    F: Fn(E1) -> E2,
    E2: Situation,
{
    MapErr {
        body,
        op,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn desc<U, E, P>(desc: E::Description, body: P) -> Describe<U, E, P>
where
    U: Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<U, E>,
{
    Describe {
        body,
        desc,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn desc_with<U, E, P, F>(f: F, body: P) -> DescribeWith<U, E, P, F>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(&E) -> E::Description,
{
    DescribeWith {
        body,
        f,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn unwrap<U, E, P>(body: P) -> Unwrap<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
{
    Unwrap {
        body,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn unwrap_or<U, E, P>(default: P::Captured, body: P) -> UnwrapOr<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Clone,
{
    UnwrapOr {
        body,
        default,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn unwrap_or_default<U, E, P>(body: P) -> UnwrapOrDefault<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Default,
{
    UnwrapOrDefault {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
#[doc(alias = "and_then")]
pub const fn complex<U, E, P, Q>(body: P, then: Q) -> Complex<U, E, P, Q>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    Q: Pattern<U, E>,
{
    Complex {
        body,
        then,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Map<U, E, P, F, Out>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(P::Captured) -> Out,
{
    body: P,
    op: F,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P, F, Out> Pattern<U, E> for Map<U, E, P, F, Out>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(P::Captured) -> Out,
{
    type Captured = Out;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        (self.op)(self.body.extract(slice, entry))
    }
}

//------------------------------------------------------------------------------

pub struct MapErr<U, E1, P, F, E2>
where
    U: Slice,
    E1: Situation,
    P: Pattern<U, E1>,
    F: Fn(E1) -> E2,
    E2: Situation,
{
    body: P,
    op: F,
    phantom: PhantomData<(U, E1, E2)>,
}

impl<U, E1, P, F, E2> Pattern<U, E2> for MapErr<U, E1, P, F, E2>
where
    U: Slice,
    E1: Situation,
    P: Pattern<U, E1>,
    F: Fn(E1) -> E2,
    E2: Situation,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E2> {
        self.body.precede(slice, entry, eof).map_err(|e| (self.op)(e))
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Describe<U, E, P>
where
    U: Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<U, E>,
{
    body: P,
    desc: E::Description,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern<U, E> for Describe<U, E, P>
where
    U: Slice,
    E: Situation,
    E::Description: Clone,
    P: Pattern<U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body
            .precede(slice, entry, eof)
            .map_err(|e| e.describe(self.desc.clone()))
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct DescribeWith<U, E, P, F>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(&E) -> E::Description,
{
    body: P,
    f: F,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P, F> Pattern<U, E> for DescribeWith<U, E, P, F>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(&E) -> E::Description,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body.precede(slice, entry, eof).map_err(|e| {
            let desc = (self.f)(&e);
            e.describe(desc)
        })
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Unwrap<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
{
    body: P,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern<U, E> for Unwrap<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        Ok(self.body.precede(slice, entry, eof).expect("unexpected input"))
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct UnwrapOr<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Clone,
{
    body: P,
    default: P::Captured,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern<U, E> for UnwrapOr<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Clone,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match entry {
            None => Ok(0),
            Some(state) => {
                let res = self.body.precede(slice, state, eof); // TODO: optimization?
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
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => self.default.clone(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct UnwrapOrDefault<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Default,
{
    body: P,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P> Pattern<U, E> for UnwrapOrDefault<U, E, P>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    P::Captured: Default,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match entry {
            None => Ok(0),
            Some(state) => {
                let res = self.body.precede(slice, state, eof);
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
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => Default::default(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct Complex<U, E, P, Q>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    Q: Pattern<U, E>,
{
    body: P,
    then: Q,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P, Q> Pattern<U, E> for Complex<U, E, P, Q>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    Q: Pattern<U, E>,
{
    type Captured = Q::Captured;
    type Internal = Alt2<P::Internal, Q::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Alt2::Var1(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let Alt2::Var1(state) = entry else {
            panic!("contract violation")
        };
        let len = self.body.precede(slice, state, eof)?;

        *entry = Alt2::Var2(self.then.init());

        let Alt2::Var2(state) = entry else { unreachable!() };
        self.then.precede(slice.split_at(len).0, state, true)
    }
    #[inline(always)]
    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract(slice, state)
    }
}
