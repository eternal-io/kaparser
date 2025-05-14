use crate::{
    combine::{convert, modifier},
    common::*,
    error::*,
    predicate::*,
};
use core::marker::PhantomData;
use impls::*;

pub mod bin;
pub mod def;
pub mod impls;

#[doc(inline)]
pub use crate::token_set;
#[doc(inline)]
pub use impls::{opaque, simple_opaque};

pub type ParseResult<T, E = SimpleError> = Result<T, E>;

pub trait Pattern<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init(&self) -> Self::Internal;

    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;

    //------------------------------------------------------------------------------

    #[inline(always)]
    fn parse(&self, slice: &mut &'i U) -> Result<Self::Captured, E> {
        let mut state = self.init();
        match self.advance(*slice, &mut state, true) {
            Ok(len) => {
                let (left, right) = slice.split_at(len);
                *slice = right;
                Ok(self.extract(left, state))
            }
            Err(e) => {
                if e.is_unfulfilled() {
                    panic!("implementation: pull after EOF")
                }
                Err(e)
            }
        }
    }

    #[inline(always)]
    fn full_match(&self, slice: &'i U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_reject_at(slice.len() - n),
        }
    }

    //------------------------------------------------------------------------------

    #[inline(always)]
    fn reiter<'p>(&'p self, slice: &'p mut &'i U) -> Reiter<'p, 'i, U, E, Self>
    where
        Self: Sized,
    {
        Reiter {
            body: self,
            src: slice,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn joined<'p, Q>(&'p self, sep: &'p Q, slice: &'p mut &'i U) -> Joined<'p, 'i, U, E, Self, Q>
    where
        Self: Sized,
        Q: Pattern<'i, U, E>,
    {
        Joined {
            body: self,
            sep,
            src: slice,
            end: false,
            phantom: PhantomData,
        }
    }

    //------------------------------------------------------------------------------

    #[inline(always)]
    fn converge<A>(self) -> convert::Converge<'i, U, E, Self, A>
    where
        Self: Sized,
        Self::Captured: Convergable<A>,
    {
        convert::converge(self)
    }

    #[inline(always)]
    fn filter<F>(self, pred: F) -> convert::Filter<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Captured) -> bool,
    {
        convert::filter(pred, self)
    }
    #[inline(always)]
    fn filter_map<F, T>(self, filter: F) -> convert::FilterMap<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Option<T>,
        T: 'static + Clone,
    {
        convert::filter_map(filter, self)
    }

    #[inline(always)]
    fn and_then<F, T>(self, op: F) -> convert::AndThen<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Result<T, E>,
        T: 'static + Clone,
    {
        convert::and_then(op, self)
    }
    #[inline(always)]
    fn then_some<T>(self, value: T) -> convert::ThenSome<'i, U, E, Self, T>
    where
        Self: Sized,
        T: Clone,
    {
        convert::then_some(value, self)
    }

    #[inline(always)]
    fn complex<Q>(self, then: Q) -> convert::Complex<'i, U, E, Self, Q>
    where
        Self: Sized,
        Q: Pattern<'i, U, E>,
    {
        convert::complex(self, then)
    }

    #[inline(always)]
    fn map<F, T>(self, op: F) -> convert::Map<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> T,
    {
        convert::map(op, self)
    }
    #[inline(always)]
    fn map_err<F, E2>(self, op: F) -> convert::MapErr<'i, U, E, Self, F, E2>
    where
        Self: Sized,
        F: Fn(E) -> E2,
        E2: Situation,
    {
        convert::map_err(op, self)
    }

    #[inline(always)]
    fn expect(self, msg: &'static str) -> convert::Expect<'i, U, E, Self>
    where
        Self: Sized,
    {
        convert::expect(msg, self)
    }
    #[inline(always)]
    fn unwrap(self) -> convert::Unwrap<'i, U, E, Self>
    where
        Self: Sized,
    {
        convert::unwrap(self)
    }

    #[inline(always)]
    fn unwrap_or(self, default: Self::Captured) -> convert::UnwrapOr<'i, U, E, Self>
    where
        Self: Sized,
        Self::Captured: Clone,
    {
        convert::unwrap_or(default, self)
    }
    #[inline(always)]
    fn unwrap_or_else<F>(self, f: F) -> convert::UnwrapOrElse<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn() -> Self::Captured,
    {
        convert::unwrap_or_else(f, self)
    }
    #[inline(always)]
    fn unwrap_or_default(self) -> convert::UnwrapOrDefault<'i, U, E, Self>
    where
        Self: Sized,
        Self::Captured: Default,
    {
        convert::unwrap_or_default(self)
    }

    //------------------------------------------------------------------------------

    #[inline(always)]
    fn parallel(self) -> modifier::Parallel<'i, U, E, Self>
    where
        Self: Sized,
    {
        modifier::parallel(self)
    }

    #[inline(always)]
    fn trace<I>(self, info: I) -> modifier::Trace<'i, U, E, Self, I>
    where
        Self: Sized,
        I: core::fmt::Display,
    {
        modifier::trace(info, self)
    }

    #[inline(always)]
    fn desc(self, desc: E::Description) -> modifier::Describe<'i, U, E, Self>
    where
        Self: Sized,
        E::Description: Clone,
    {
        modifier::desc(desc, self)
    }
    #[inline(always)]
    fn desc_with<F>(self, f: F) -> modifier::DescribeWith<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&E) -> E::Description,
    {
        modifier::desc_with(f, self)
    }
}

//==================================================================================================

impl<'i, U, E> Pattern<'i, U, E> for &'i U
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if slice.len() < self.len() {
            match eof {
                true => E::raise_reject_at(slice.len()),
                false => E::raise_unfulfilled(Some((self.len() - slice.len()).try_into().unwrap())),
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
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.len()).0
    }
}

impl<'i, U, E, P> Pattern<'i, U, E> for [P; 1]
where
    U: ?Sized + Slice,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = U::Item;
    type Internal = ();

    #[inline(always)]
    fn init(&self) -> Self::Internal {}

    #[inline(always)]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.first().unwrap()
    }
}

//==================================================================================================

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn slice() {
        let pat = simple_opaque("");
        assert!(pat.full_match("").is_ok());
        assert_eq!(pat.full_match("?").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("??").unwrap_err().offset(), 0);

        let pat = simple_opaque("A");
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("A").unwrap(), "A");
        assert_eq!(pat.full_match("AA").unwrap_err().offset(), 1);

        let pat = simple_opaque("AB");
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("AB").unwrap(), "AB");
        assert_eq!(pat.full_match("ABCD").unwrap_err().offset(), 2);

        let pat = simple_opaque("ABCD");
        assert_eq!(pat.full_match("").unwrap_err().offset(), 0);
        assert_eq!(pat.full_match("AB").unwrap_err().offset(), 2);
        assert_eq!(pat.full_match("ABCD").unwrap(), "ABCD");
    }
}
