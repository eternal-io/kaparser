use super::*;
use core::fmt::Display;

#[cfg(feature = "debug")]
extern crate std;

#[inline(always)]
pub const fn igc<U>(slice: &U) -> IgnoreCase<U>
where
    U: ?Sized + SliceAscii,
{
    IgnoreCase { slice }
}

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
pub const fn trace<'i, U, E, P, I>(info: I, body: P) -> Trace<'i, U, E, P, I>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    I: Display,
{
    Trace {
        body,
        info,
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

pub struct IgnoreCase<'i, U>
where
    U: ?Sized + SliceAscii,
{
    slice: &'i U,
}

impl<'i, U, E> Pattern<'i, U, E> for IgnoreCase<'_, U>
where
    U: ?Sized + SliceAscii + 'i,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}
    #[inline(always)]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if slice.len() < self.slice.len() {
            match eof {
                true => E::raise_reject_at(slice.len()),
                false => E::raise_unfulfilled(Some((self.slice.len() - slice.len()).try_into().unwrap())),
            }
        } else {
            for ((off, expected), item) in self.slice.iter_indices().zip(slice.iter()) {
                if item != expected {
                    return E::raise_reject_at(off);
                }
            }
            Ok(self.slice.len())
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.slice.len()).0
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

#[cfg_attr(not(feature = "debug"), allow(dead_code))]
pub struct Trace<'i, U, E, P, I>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    I: Display,
{
    body: P,
    info: I,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, I> Pattern<'i, U, E> for Trace<'i, U, E, P, I>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    I: Display,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        #[cfg(feature = "debug")]
        std::println!("{}", self.info);
        self.body.init()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof)
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
