use crate::{
    combine::{control, convert},
    common::*,
    error::*,
    predicate::*,
};

#[doc(inline)]
pub use crate::token_set;

pub trait Pattern<U, E>
where
    U: Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init(&self) -> Self::Internal;

    fn precede(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E>;

    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured;

    #[inline(always)]
    fn verify<F>(self, f: F) -> control::Verify<U, E, Self, F>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> bool,
    {
        control::verify(f, self)
    }

    #[inline(always)]
    fn map<F, Out>(self, op: F) -> convert::Map<U, E, Self, F, Out>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Out,
    {
        convert::map(op, self)
    }
    #[inline(always)]
    fn map_err<F, E2>(self, op: F) -> convert::MapErr<U, E, Self, F, E2>
    where
        Self: Sized,
        F: Fn(E) -> E2,
        E2: Situation,
    {
        convert::map_err(op, self)
    }

    #[inline(always)]
    fn desc(self, desc: E::Description) -> convert::Describe<U, E, Self>
    where
        Self: Sized,
        E::Description: Clone,
    {
        convert::desc(desc, self)
    }
    #[inline(always)]
    fn desc_with<F>(self, f: F) -> convert::DescribeWith<U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&E) -> E::Description,
    {
        convert::desc_with(f, self)
    }

    #[inline(always)]
    fn unwrap(self) -> convert::Unwrap<U, E, Self>
    where
        Self: Sized,
    {
        convert::unwrap(self)
    }
    #[inline(always)]
    fn unwrap_or(self, default: Self::Captured) -> convert::UnwrapOr<U, E, Self>
    where
        Self: Sized,
        Self::Captured: Clone,
    {
        convert::unwrap_or(default, self)
    }
    #[inline(always)]
    fn unwrap_or_default(self) -> convert::UnwrapOrDefault<U, E, Self>
    where
        Self: Sized,
        Self::Captured: Default,
    {
        convert::unwrap_or_default(self)
    }

    #[inline(always)]
    fn complex<Q>(self, then: Q) -> convert::Complex<U, E, Self, Q>
    where
        Self: Sized,
        Q: Pattern<U, E>,
    {
        convert::complex(self, then)
    }
}

//------------------------------------------------------------------------------

impl<U, E> Pattern<U, E> for U
where
    U: Slice,
    E: Situation,
{
    type Captured = U;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: U, _ntry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        if slice.len() < self.len() {
            match eof {
                true => E::raise_unfulfilled(Some((self.len() - slice.len()).try_into().unwrap())),
                false => E::raise_reject_at(slice.len()),
            }
        } else {
            for ((off, expected), item) in self.iter_indices().zip(slice.iter()) {
                if item != expected {
                    return E::raise_reject_at(off);
                }
            }
            Ok(self.len())
        }
    }

    #[inline(always)]
    fn extract(&self, slice: U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.len()).0
    }
}

impl<U, E, P> Pattern<U, E> for [P; 1]
where
    U: Slice,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = U::Item;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn precede(&self, slice: U, _ntry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        match slice.first() {
            Some(item) => match self[0].predicate(&item) {
                true => Ok(slice.len_of(item)),
                false => E::raise_reject_at(0),
            },
            None => match eof {
                true => E::raise_reject_at(0),
                false => E::raise_unfulfilled(None),
            },
        }
    }

    #[inline(always)]
    fn extract(&self, slice: U, _ntry: Self::Internal) -> Self::Captured {
        slice.first().unwrap()
    }
}
