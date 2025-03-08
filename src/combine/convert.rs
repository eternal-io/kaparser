use super::*;

#[inline(always)]
pub const fn and<'i, U, E, P, T>(t: T, body: P) -> And<'i, U, E, P, T>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    And {
        body,
        and: t,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn map<'i, U, E, P, F, Out>(op: F, body: P) -> Map<'i, U, E, P, F, Out>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Out,
{
    Map {
        body,
        op,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn map_err<'i, U, E1, P, F, E2>(op: F, body: P) -> MapErr<'i, U, E1, P, F, E2>
where
    U: ?Sized + Slice,
    E1: Situation,
    P: Pattern<'i, U, E1>,
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

#[inline(always)]
pub const fn unwrap<'i, U, E, P>(body: P) -> Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Unwrap {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn or<'i, U, E, P>(default: P::Captured, body: P) -> Or<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    Or {
        body,
        default,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn or_else<'i, U, E, P, F>(f: F, body: P) -> OrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    OrElse {
        body,
        f,
        phantom: PhantomData,
    }
}
#[inline(always)]
pub const fn or_default<'i, U, E, P>(body: P) -> OrDefault<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    OrDefault {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
#[doc(alias = "and_then")]
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

//==================================================================================================

pub struct And<'i, U, E, P, T>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    body: P,
    and: T,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, T> Pattern<'i, U, E> for And<'i, U, E, P, T>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    type Captured = T;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        self.and.clone()
    }
}

//------------------------------------------------------------------------------

pub struct Map<'i, U, E, P, F, Out>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Out,
{
    body: P,
    op: F,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, F, Out> Pattern<'i, U, E> for Map<'i, U, E, P, F, Out>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Out,
{
    type Captured = Out;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.op)(self.body.extract(slice, entry))
    }
}

//------------------------------------------------------------------------------

pub struct MapErr<'i, U, E1, P, F, E2>
where
    U: ?Sized + Slice,
    E1: Situation,
    P: Pattern<'i, U, E1>,
    F: Fn(E1) -> E2,
    E2: Situation,
{
    body: P,
    op: F,
    phantom: PhantomData<(&'i U, E1, E2)>,
}

impl<'i, U, E1, P, F, E2> Pattern<'i, U, E2> for MapErr<'i, U, E1, P, F, E2>
where
    U: ?Sized + Slice,
    E1: Situation,
    P: Pattern<'i, U, E1>,
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
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E2> {
        self.body.precede(slice, entry, eof).map_err(|e| (self.op)(e))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
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
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body
            .precede(slice, entry, eof)
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
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.precede(slice, entry, eof).map_err(|e| {
            let desc = (self.f)(&e);
            e.describe(desc)
        })
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        Ok(self.body.precede(slice, entry, eof).expect("unexpected input"))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Or<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    body: P,
    default: P::Captured,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Or<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => self.default.clone(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct OrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    body: P,
    f: F,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, F> Pattern<'i, U, E> for OrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => (self.f)(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

//------------------------------------------------------------------------------

pub struct OrDefault<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for OrDefault<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => Default::default(),
            Some(state) => self.body.extract(slice, state),
        }
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
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let Alt2::Var1(state) = entry else {
            panic!("contract violation")
        };
        let len = self.body.precede(slice, state, eof)?;

        *entry = Alt2::Var2(self.then.init());

        let Alt2::Var2(state) = entry else { unreachable!() };
        self.then.precede(slice.split_at(len).0, state, true)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract(slice, state)
    }
}
