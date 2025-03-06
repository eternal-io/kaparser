use crate::{combine::convert, common::*, error::*, predicate::*};

#[doc(inline)]
pub use crate::token_set;

pub fn __pat<'i, U, Cap, E>(pat: impl Pattern<'i, U, E, Captured = Cap>) -> impl Pattern<'i, U, E, Captured = Cap>
where
    U: ?Sized + Slice,
    E: Situation,
{
    pat
}

pub trait Pattern<'i, U, E>
where
    U: ?Sized + Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init(&self) -> Self::Internal;

    fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;

    //------------------------------------------------------------------------------

    #[inline(always)]
    fn parse(&self, slice: &mut &'i U) -> Result<Self::Captured, E> {
        let mut state = self.init();
        match self.precede(*slice, &mut state, true) {
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
    fn parse_peek(&self, slice: &'i U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        self.parse(&mut sli)
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
    fn and<T>(self, t: T) -> convert::And<'i, U, E, Self, T>
    where
        Self: Sized,
        T: Clone,
    {
        convert::and(t, self)
    }
    #[inline(always)]
    fn map<F, Out>(self, op: F) -> convert::Map<'i, U, E, Self, F, Out>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Out,
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
    fn desc(self, desc: E::Description) -> convert::Describe<'i, U, E, Self>
    where
        Self: Sized,
        E::Description: Clone,
    {
        convert::desc(desc, self)
    }
    #[inline(always)]
    fn desc_with<F>(self, f: F) -> convert::DescribeWith<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&E) -> E::Description,
    {
        convert::desc_with(f, self)
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
    fn unwrap_or_default(self) -> convert::UnwrapOrDefault<'i, U, E, Self>
    where
        Self: Sized,
        Self::Captured: Default,
    {
        convert::unwrap_or_default(self)
    }

    #[inline(always)]
    fn complex<Q>(self, then: Q) -> convert::Complex<'i, U, E, Self, Q>
    where
        Self: Sized,
        Q: Pattern<'i, U, E>,
    {
        convert::complex(self, then)
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
    fn precede(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn precede(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
        let pat = __pat::<_, _, ParseError>("");
        assert!(pat.full_match("").is_ok());
        assert_eq!(pat.full_match("?").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("??").unwrap_err().length(), 0);

        let pat = __pat::<_, _, ParseError>("A");
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("A").unwrap(), "A");
        assert_eq!(pat.full_match("AA").unwrap_err().length(), 1);

        let pat = __pat::<_, _, ParseError>("AB");
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("AB").unwrap(), "AB");
        assert_eq!(pat.full_match("ABCD").unwrap_err().length(), 2);

        let pat = __pat::<_, _, ParseError>("ABCD");
        assert_eq!(pat.full_match("").unwrap_err().length(), 0);
        assert_eq!(pat.full_match("AB").unwrap_err().length(), 2);
        assert_eq!(pat.full_match("ABCD").unwrap(), "ABCD");
    }
}
