use super::*;

#[inline(always)]
pub const fn map<'i, U, P, F, O>(f: F, body: P) -> Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
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
pub const fn complex<'i, U, P, Q>(body: P, then: Q) -> Complex<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    Q: Pattern<'i, U>,
{
    Complex {
        body,
        then,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> O,
{
    body: P,
    then: F,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, F, O> Pattern<'i, U> for Map<'i, U, P, F, O>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> O,
{
    type Captured = O;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        self.body.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.then)(self.body.extract(slice, entry))
    }
}

//------------------------------------------------------------------------------

pub struct Complex<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    Q: Pattern<'i, U>,
{
    body: P,
    then: Q,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, Q> Pattern<'i, U> for Complex<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    Q: Pattern<'i, U>,
{
    type Captured = Q::Captured;
    type Internal = Alt2<P::Internal, Q::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Alt2::Var1(self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        if let Alt2::Var1(state) = entry {
            let (t, len) = self.body.precede(slice, state, eof)?;
            if !t.is_accepted() {
                return Some((t, len));
            }

            *entry = Alt2::Var2(self.then.init());
            let Alt2::Var2(state) = entry else { unreachable!() };
            Some(
                self.then
                    .precede(slice.split_at(len).0, state, true)
                    .expect("implementation: pull after EOF"),
            )
        } else {
            panic!("contract violation")
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let Alt2::Var2(state) = entry else {
            panic!("contract violation")
        };
        self.then.extract(slice, state)
    }
}
