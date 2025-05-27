use crate::{
    // combine::{convert, modifier},
    common::*,
    error::*,
    predicate::*,
};

// pub mod bin;
// pub mod def;
// pub mod impls;

#[doc(inline)]
pub use crate::token_set;

pub type ParseResult<T, E = SimpleError> = Result<T, E>;

#[inline]
pub const fn opaque<U, E, Cap>(pattern: impl Pattern<U, E, Captured = Cap>) -> impl Pattern<U, E, Captured = Cap>
where
    U: Slice,
    E: Situation,
{
    pattern
}
#[inline]
pub const fn opaque_simple<U, Cap>(
    pattern: impl Pattern<U, SimpleError, Captured = Cap>,
) -> impl Pattern<U, SimpleError, Captured = Cap>
where
    U: Slice,
{
    pattern
}

//==================================================================================================

pub trait Pattern<U, E>
where
    U: Slice,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init(&self) -> Self::Internal;

    fn advance(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured;

    fn inject_base_off(&self, entry: &mut Self::Internal, base_off: usize) {
        let _ = (entry, base_off);
    }

    //------------------------------------------------------------------------------

    #[inline]
    fn parse(&self, slice: &mut dyn DynamicSlice<U>) -> Result<Self::Captured, E> {
        let mut state = self.init();
        match self.advance(slice.rest(), &mut state, true) {
            Err(e) if e.is_unfulfilled() => panic!("implementation: pull after EOF"),
            Err(e) => Err(e.backtrack(slice.consumed())),
            Ok(len) => {
                self.inject_base_off(&mut state, slice.consumed());
                Ok(self.extract(slice.bump(len), state))
            }
        }
    }

    #[inline]
    fn fullmatch(&self, slice: U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_halt_at(slice.len() - n),
        }
    }

    //------------------------------------------------------------------------------

    // #[inline]
    // fn reiter<'s, 'p, S>(&'p self, slice: &'s mut S) -> impls::Reiter<'s, 'p, 'i, U, E, Self, S>
    // where
    //     Self: Sized,
    //     S: DynamicSlice<'i, U>,
    // {
    //     impls::reiter(self, slice)
    // }

    // #[inline]
    // fn joined<'s, 'p, Q, S>(&'p self, sep: &'p Q, slice: &'s mut S) -> impls::Joined<'s, 'p, 'i, U, E, Self, Q, S>
    // where
    //     Self: Sized,
    //     Q: Pattern<'i, U, E>,
    //     S: DynamicSlice<'i, U>,
    // {
    //     impls::joined(self, sep, slice)
    // }

    //------------------------------------------------------------------------------

    // #[inline]
    // fn converge<A>(self) -> convert::Converge<'i, U, E, Self, A>
    // where
    //     Self: Sized,
    //     Self::Captured: Convergable<A>,
    // {
    //     convert::converge(self)
    // }

    // #[inline]
    // fn filter<F>(self, pred: F) -> convert::Filter<'i, U, E, Self, F>
    // where
    //     Self: Sized,
    //     F: Fn(&Self::Captured) -> bool,
    // {
    //     convert::filter(pred, self)
    // }
    // #[inline]
    // fn filter_map<F, T>(self, filter: F) -> convert::FilterMap<'i, U, E, Self, F, T>
    // where
    //     Self: Sized,
    //     F: Fn(Self::Captured) -> Option<T>,
    //     T: 'static + Clone,
    // {
    //     convert::filter_map(filter, self)
    // }

    // #[inline]
    // fn and_then<F, T>(self, op: F) -> convert::AndThen<'i, U, E, Self, F, T>
    // where
    //     Self: Sized,
    //     F: Fn(Self::Captured) -> Result<T, E>,
    //     T: 'static + Clone,
    // {
    //     convert::and_then(op, self)
    // }
    // #[inline]
    // fn then_some<T>(self, value: T) -> convert::ThenSome<'i, U, E, Self, T>
    // where
    //     Self: Sized,
    //     T: Clone,
    // {
    //     convert::then_some(value, self)
    // }

    // #[inline]
    // fn complex<Q>(self, then: Q) -> convert::Complex<'i, U, E, Self, Q>
    // where
    //     Self: Sized,
    //     Q: Pattern<'i, U, E>,
    // {
    //     convert::complex(self, then)
    // }

    // #[inline]
    // fn map<F, T>(self, op: F) -> convert::Map<'i, U, E, Self, F, T>
    // where
    //     Self: Sized,
    //     F: Fn(Self::Captured) -> T,
    // {
    //     convert::map(op, self)
    // }
    // #[inline]
    // fn map_err<F, E2>(self, op: F) -> convert::MapErr<'i, U, E, Self, F, E2>
    // where
    //     Self: Sized,
    //     F: Fn(E) -> E2,
    //     E2: Situation,
    // {
    //     convert::map_err(op, self)
    // }

    // #[inline]
    // fn expect(self, msg: &'static str) -> convert::Expect<'i, U, E, Self>
    // where
    //     Self: Sized,
    // {
    //     convert::expect(msg, self)
    // }
    // #[inline]
    // fn unwrap(self) -> convert::Unwrap<'i, U, E, Self>
    // where
    //     Self: Sized,
    // {
    //     convert::unwrap(self)
    // }

    // #[inline]
    // fn unwrap_or(self, default: Self::Captured) -> convert::UnwrapOr<'i, U, E, Self>
    // where
    //     Self: Sized,
    //     Self::Captured: Clone,
    // {
    //     convert::unwrap_or(default, self)
    // }
    // #[inline]
    // fn unwrap_or_else<F>(self, f: F) -> convert::UnwrapOrElse<'i, U, E, Self, F>
    // where
    //     Self: Sized,
    //     F: Fn() -> Self::Captured,
    // {
    //     convert::unwrap_or_else(f, self)
    // }
    // #[inline]
    // fn unwrap_or_default(self) -> convert::UnwrapOrDefault<'i, U, E, Self>
    // where
    //     Self: Sized,
    //     Self::Captured: Default,
    // {
    //     convert::unwrap_or_default(self)
    // }

    //------------------------------------------------------------------------------

    // #[inline]
    // fn parallel(self) -> modifier::Parallel<'i, U, E, Self>
    // where
    //     Self: Sized,
    // {
    //     modifier::parallel(self)
    // }

    // #[inline]
    // fn trace<I>(self, info: I) -> modifier::Trace<'i, U, E, Self, I>
    // where
    //     Self: Sized,
    //     I: core::fmt::Display,
    // {
    //     modifier::trace(info, self)
    // }

    // #[inline]
    // fn desc(self, desc: E::Description) -> modifier::Describe<'i, U, E, Self>
    // where
    //     Self: Sized,
    //     E::Description: Clone,
    // {
    //     modifier::desc(desc, self)
    // }
    // #[inline]
    // fn desc_with<F>(self, f: F) -> modifier::DescribeWith<'i, U, E, Self, F>
    // where
    //     Self: Sized,
    //     F: Fn(&E) -> E::Description,
    // {
    //     modifier::desc_with(f, self)
    // }

    // #[inline]
    // fn void(self) -> modifier::Void<'i, U, E, Self>
    // where
    //     Self: Sized,
    // {
    //     modifier::void(self)
    // }
}

//==================================================================================================

impl<T, U, Ue, E> Pattern<U, E> for Ue
where
    T: PartialEq,
    U: Slice<Item = T>,
    Ue: Slice<Item = T>,
    E: Situation,
{
    type Captured = U;
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}

    #[inline]
    fn advance(&self, slice: U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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

    #[inline]
    fn extract(&self, slice: U, _ntry: Self::Internal) -> Self::Captured {
        slice.before(self.len())
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

    #[inline]
    fn init(&self) -> Self::Internal {}

    #[inline]
    fn advance(&self, slice: U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        match slice.first() {
            Some(item) => match self[0].predicate(&item) {
                true => Ok(U::len_of(item)),
                false => E::raise_reject_at(0),
            },
            None => match eof {
                true => E::raise_reject_at(0),
                false => E::raise_unfulfilled(None),
            },
        }
    }

    #[inline]
    fn extract(&self, slice: U, _ntry: Self::Internal) -> Self::Captured {
        slice.first().unwrap()
    }
}

//==================================================================================================

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::string::String;

    #[test]
    fn slice() {
        let pat = opaque_simple("");
        assert!(pat.fullmatch("").is_ok());
        assert_eq!(pat.fullmatch("?").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("??").unwrap_err().offset(), 0);

        let pat = opaque_simple("A");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("A").unwrap(), "A");
        assert_eq!(pat.fullmatch("AA").unwrap_err().offset(), 1);

        let pat = opaque_simple("AB");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("AB").unwrap(), "AB");
        assert_eq!(pat.fullmatch("ABCD").unwrap_err().offset(), 2);

        let pat = opaque_simple("ABCD");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("AB").unwrap_err().offset(), 2);
        assert_eq!(pat.fullmatch("ABCD").unwrap(), "ABCD");
    }

    #[test]
    fn test_lifetime() {
        let pat = opaque_simple("foobar");

        const MSG: &'static str = "foobar";
        let msging = String::from("foobar");
        let msg = msging.as_str();

        assert!(pat.fullmatch(MSG).is_ok());
        assert!(pat.fullmatch(msg).is_ok());
    }

    // -------------

    trait SmallSlice: Sized {
        // or: type Slice;
        //     fn _(..) -> (Self::Slice, Self::Slice), but require delegate too many.
        fn split_at(&self, mid: usize) -> (Self, Self);
    }

    impl<'i> SmallSlice for &'i str {
        fn split_at(&self, mid: usize) -> (Self, Self) {
            (*self).split_at(mid)
        }
    }

    trait SmallPattern<U: SmallSlice> {
        type Captured;

        fn parse(&self, slice: U) -> Self::Captured;
    }

    impl<T, U: SmallSlice> SmallPattern<U> for T {
        type Captured = U;

        fn parse(&self, slice: U) -> Self::Captured {
            slice.split_at(0).0
        }
    }

    #[test]
    fn test_lifetime2() {
        const MSG: &'static str = "foobar";
        let msging = String::from("foobar");
        let msg = msging.as_str();

        ().parse(MSG);
        ().parse(msg);
    }
}
