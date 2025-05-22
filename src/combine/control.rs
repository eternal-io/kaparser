use super::*;

pub struct EOF;
pub struct Halt;
pub struct Reject;
pub struct TODO;

#[inline]
pub const fn igc<U>(slice: &U) -> IgnoreCase<U>
where
    U: ?Sized + ThinSlice,
{
    IgnoreCase { slice }
}

#[inline]
pub const fn cut<'i, U, E, P>(body: P) -> Cut<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Cut {
        body,
        phantom: PhantomData,
    }
}

#[inline]
pub const fn cond<'i, U, E, P>(b: bool, body: P) -> Cond<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Cond {
        cond: b,
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

impl<'i, U, E> Pattern<'i, U, E> for EOF
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured = ();
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match slice.is_empty() && eof {
            true => Ok(0),
            false => E::raise_reject_at(0),
        }
    }
    #[inline]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {}
}

impl<'i, U, E> Pattern<'i, U, E> for Halt
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured = ();
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, _lice: &U, _ntry: &mut Self::Internal, _of: bool) -> Result<usize, E> {
        E::raise_halt_at(0)
    }
    #[inline]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {}
}

impl<'i, U, E> Pattern<'i, U, E> for Reject
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured = ();
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, _lice: &U, _ntry: &mut Self::Internal, _of: bool) -> Result<usize, E> {
        E::raise_reject_at(0)
    }
    #[inline]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {}
}

impl<'i, U, E> Pattern<'i, U, E> for TODO
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured = ();
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, _lice: &U, _ntry: &mut Self::Internal, _of: bool) -> Result<usize, E> {
        panic!("not yet implemented")
    }
    #[inline]
    fn extract(&self, _lice: &'i U, _ntry: Self::Internal) -> Self::Captured {}
}

//------------------------------------------------------------------------------

pub struct IgnoreCase<'i, U>
where
    U: ?Sized + ThinSlice,
{
    slice: &'i U,
}

impl<'i, U, E> Pattern<'i, U, E> for IgnoreCase<'_, U>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}
    #[inline]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if slice.len() < self.slice.len() {
            match eof {
                true => E::raise_reject_at(slice.len()),
                false => E::raise_unfulfilled(Some((self.slice.len() - slice.len()).try_into().unwrap())),
            }
        } else {
            for ((off, expected), item) in self.slice.iter_indices().zip(slice.iter()) {
                if !U::eq_ignore_ascii_case(item, expected) {
                    return E::raise_reject_at(off);
                }
            }
            Ok(self.slice.len())
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.slice.len()).0
    }
}

//------------------------------------------------------------------------------

pub struct Cut<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Cut<'i, U, E, P>
where
    U: ?Sized + Slice,
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
        self.body.advance(slice, entry, eof).map_err(Situation::cut)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Cond<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    cond: bool,
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Cond<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match self.cond {
            true => self.body.advance(slice, entry, eof),
            false => Ok(0),
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        match self.cond {
            true => Some(self.body.extract(slice, entry)),
            false => None,
        }
    }
}
