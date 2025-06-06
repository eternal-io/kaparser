use super::*;
use core::mem;

#[inline]
pub const fn converge<'i, U, E, P, A>(body: P) -> Converge<'i, U, E, P, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Convergable<A>,
{
    Converge {
        body,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn filter<'i, U, E, P, F>(pred: F, body: P) -> Filter<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&P::Captured) -> bool,
{
    Filter {
        body,
        pred,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn filter_map<'i, U, E, P, F, T>(filter: F, body: P) -> FilterMap<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Option<T>,
    T: 'static + Clone,
{
    FilterMap {
        body,
        filter,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn and_then<'i, U, E, P, F, T>(op: F, body: P) -> AndThen<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Result<T, E>,
    T: 'static + Clone,
{
    AndThen {
        body,
        op,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn then_some<'i, U, E, P, T>(value: T, body: P) -> ThenSome<'i, U, E, P, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    ThenSome {
        body,
        value,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn complex<'i, U, E, P, Q>(body: P, then: Q) -> Complex<'i, U, E, P, Q>
where
    U: ?Sized + Slice + 'i,
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

#[inline]
pub const fn map<'i, U, E, P, F, T>(op: F, body: P) -> Map<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> T,
{
    Map {
        body,
        op,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn map_err<'i, U, E1, P, F, E2>(op: F, body: P) -> MapErr<'i, U, E1, P, F, E2>
where
    U: ?Sized + Slice + 'i,
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

#[inline]
pub const fn expect<'i, U, E, P>(msg: &'static str, body: P) -> Expect<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Expect {
        body,
        msg,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn unwrap<'i, U, E, P>(body: P) -> Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Unwrap {
        body,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn unwrap_or<'i, U, E, P>(default: P::Captured, body: P) -> UnwrapOr<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    UnwrapOr {
        body,
        default,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn unwrap_or_else<'i, U, E, P, F>(f: F, body: P) -> UnwrapOrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    UnwrapOrElse {
        body,
        f,
        phantom: PhantomData,
    }
}
#[inline]
pub const fn unwrap_or_default<'i, U, E, P>(body: P) -> UnwrapOrDefault<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    UnwrapOrDefault {
        body,
        phantom: PhantomData,
    }
}

//==================================================================================================

pub struct Converge<'i, U, E, P, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Convergable<A>,
{
    body: P,
    phantom: PhantomData<(&'i U, E, A)>,
}
impl<'i, U, E, P, A> Pattern<'i, U, E> for Converge<'i, U, E, P, A>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Convergable<A>,
{
    type Captured = A;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry).converge()
    }
}

//--------------------------------------------------------------------------------------------------

pub struct Filter<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&P::Captured) -> bool,
{
    body: P,
    pred: F,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, F> Pattern<'i, U, E> for Filter<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(&P::Captured) -> bool,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    #[allow(unsafe_code)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let offset = self.body.advance(slice, entry, eof)?;

        // SAFETY: The captured is only used temporarily in this function.
        //         No leaks can occur without internal mutability.
        // match unsafe { (self.pred)(&self.body.extract(mem::transmute::<&U, &'i U>(slice), entry.clone())) } {
        //     true => Ok(offset),
        //     false => E::raise_reject_at(0),
        // }
        todo!()
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

pub struct FilterMap<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Option<T>,
    T: 'static + Clone,
{
    body: P,
    filter: F,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, F, T> Pattern<'i, U, E> for FilterMap<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Option<T>,
    T: 'static + Clone,
{
    type Captured = T;
    type Internal = Alt3<P::Internal, (), T>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Alt3::Var2(())
    }
    #[inline]
    #[allow(unsafe_code)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if !matches!(entry, Alt3::Var1(_)) {
            *entry = Alt3::Var1(self.body.init());
        }

        let Alt3::Var1(state) = entry else { unreachable!() };
        let offset = self.body.advance(slice, state, eof)?;

        let Alt3::Var1(state) = mem::replace(entry, Alt3::Var2(())) else {
            unreachable!()
        };

        // SAFETY: The captured is only used temporarily in this function.
        //         `T` is `'static` that outlives `'i`. No leaks can occur without internal mutability.
        *entry = Alt3::Var3(
            (self.filter)(self.body.extract(unsafe { mem::transmute::<&U, &'i U>(slice) }, state))
                .ok_or_else(|| E::reject_at(0))?,
        );

        Ok(offset)
    }
    #[inline]
    fn extract(&self, _lice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt3::Var3(output) = entry else {
            panic!("contract violation")
        };
        output
    }
}

//--------------------------------------------------------------------------------------------------

pub struct AndThen<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Result<T, E>,
    T: 'static + Clone,
{
    body: P,
    op: F,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, F, T> Pattern<'i, U, E> for AndThen<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> Result<T, E>,
    T: 'static + Clone,
{
    type Captured = T;
    type Internal = Alt3<P::Internal, (), T>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Alt3::Var2(())
    }
    #[inline]
    #[allow(unsafe_code)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if !matches!(entry, Alt3::Var1(_)) {
            *entry = Alt3::Var1(self.body.init());
        }

        let Alt3::Var1(state) = entry else { unreachable!() };
        let offset = self.body.advance(slice, state, eof)?;

        let Alt3::Var1(state) = mem::replace(entry, Alt3::Var2(())) else {
            unreachable!()
        };

        // SAFETY: The captured is only used temporarily in this function.
        //         `T` is `'static` that outlives `'i`. No leaks can occur without internal mutability.
        *entry = Alt3::Var3((self.op)(
            self.body.extract(unsafe { mem::transmute::<&U, &'i U>(slice) }, state),
        )?);

        Ok(offset)
    }
    #[inline]
    fn extract(&self, _lice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt3::Var3(output) = entry else {
            panic!("contract violation")
        };
        output
    }
}

pub struct ThenSome<'i, U, E, P, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    body: P,
    value: T,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, T> Pattern<'i, U, E> for ThenSome<'i, U, E, P, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    T: Clone,
{
    type Captured = T;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        self.value.clone()
    }
}

//--------------------------------------------------------------------------------------------------

pub struct Complex<'i, U, E, P, Q>
where
    U: ?Sized + Slice + 'i,
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
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    type Captured = Q::Captured;
    type Internal = Alt2<P::Internal, Q::Internal>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Alt2::Var1(self.body.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let Alt2::Var1(state) = entry else {
            panic!("contract violation")
        };
        let len = self.body.advance(slice, state, eof)?;

        *entry = Alt2::Var2(self.then.init());

        let Alt2::Var2(state) = entry else { unreachable!() };
        self.then.advance(slice.before(len), state, true)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract(slice, state)
    }
}

//--------------------------------------------------------------------------------------------------

pub struct Map<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> T,
{
    body: P,
    op: F,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, F, T> Pattern<'i, U, E> for Map<'i, U, E, P, F, T>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn(P::Captured) -> T,
{
    type Captured = T;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.op)(self.body.extract(slice, entry))
    }
}

pub struct MapErr<'i, U, E1, P, F, E2>
where
    U: ?Sized + Slice + 'i,
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
    U: ?Sized + Slice + 'i,
    E1: Situation,
    P: Pattern<'i, U, E1>,
    F: Fn(E1) -> E2,
    E2: Situation,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E2> {
        self.body.advance(slice, entry, eof).map_err(|e| (self.op)(e))
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//--------------------------------------------------------------------------------------------------

pub struct Expect<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    msg: &'static str,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P> Pattern<'i, U, E> for Expect<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        Ok(self.body.advance(slice, entry, eof).expect(self.msg))
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

pub struct Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P> Pattern<'i, U, E> for Unwrap<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        Ok(self.body.advance(slice, entry, eof).expect("unexpected input"))
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//--------------------------------------------------------------------------------------------------

pub struct UnwrapOr<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    body: P,
    default: P::Captured,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P> Pattern<'i, U, E> for UnwrapOr<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Clone,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let res = self
            .body
            .advance(slice, entry.as_mut().expect("contract violation"), eof);
        if let Err(e) = &res {
            if !e.is_unfulfilled() {
                *entry = None;
            }
        }
        res
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => self.default.clone(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

pub struct UnwrapOrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    body: P,
    f: F,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P, F> Pattern<'i, U, E> for UnwrapOrElse<'i, U, E, P, F>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    F: Fn() -> P::Captured,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let res = self
            .body
            .advance(slice, entry.as_mut().expect("contract violation"), eof);
        if let Err(e) = &res {
            if !e.is_unfulfilled() {
                *entry = None;
            }
        }
        res
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => (self.f)(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}

pub struct UnwrapOrDefault<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}
impl<'i, U, E, P> Pattern<'i, U, E> for UnwrapOrDefault<'i, U, E, P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    P::Captured: Default,
{
    type Captured = P::Captured;
    type Internal = Option<P::Internal>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Some(self.body.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let res = self
            .body
            .advance(slice, entry.as_mut().expect("contract violation"), eof);
        if let Err(e) = &res {
            if !e.is_unfulfilled() {
                *entry = None;
            }
        }
        res
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match entry {
            None => Default::default(),
            Some(state) => self.body.extract(slice, state),
        }
    }
}
