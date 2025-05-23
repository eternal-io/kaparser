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

pub type ParseResult<T, E = SimpleError> = Result<T, E>;

pub trait Pattern<'i, U, E>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init(&self) -> Self::Internal;

    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;

    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;

    //------------------------------------------------------------------------------

    #[inline]
    fn parse(&self, slice: &mut dyn AdvanceSlice<'i, U>) -> Result<Self::Captured, E> {
        let mut state = self.init();
        match self.advance(slice.rest(), &mut state, true) {
            Ok(len) => Ok(self.extract(slice.bump(len), state)),
            Err(e) if e.is_unfulfilled() => panic!("implementation: pull after EOF"),
            Err(e) => Err(e.backtrack(slice.consumed())),
        }
    }

    #[inline]
    fn fullmatch(&self, slice: &'i U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_halt_at(slice.len() - n),
        }
    }

    //------------------------------------------------------------------------------

    #[inline]
    fn opaque<Ui, Ei, Cap>(self) -> impl Pattern<'i, Ui, Ei, Captured = Cap>
    where
        Self: Sized + Pattern<'i, Ui, Ei, Captured = Cap>,
        Ui: ?Sized + Slice + 'i,
        Ei: Situation,
    {
        self
    }

    #[inline]
    fn opaque_simple<Ui, Cap>(self) -> impl Pattern<'i, Ui, SimpleError, Captured = Cap>
    where
        Self: Sized + Pattern<'i, Ui, SimpleError, Captured = Cap>,
        Ui: ?Sized + Slice + 'i,
    {
        self
    }

    //------------------------------------------------------------------------------

    #[inline]
    fn reiter<'s, 'p, S>(&'p self, slice: &'s mut S) -> Reiter<'s, 'p, 'i, U, E, Self, S>
    where
        Self: Sized,
        S: AdvanceSlice<'i, U>,
    {
        Reiter {
            body: self,
            src: slice,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn joined<'s, 'p, Q, S>(&'p self, sep: &'p Q, slice: &'s mut S) -> Joined<'s, 'p, 'i, U, E, Self, Q, S>
    where
        Self: Sized,
        Q: Pattern<'i, U, E>,
        S: AdvanceSlice<'i, U>,
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

    #[inline]
    fn converge<A>(self) -> convert::Converge<'i, U, E, Self, A>
    where
        Self: Sized,
        Self::Captured: Convergable<A>,
    {
        convert::converge(self)
    }

    #[inline]
    fn filter<F>(self, pred: F) -> convert::Filter<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Captured) -> bool,
    {
        convert::filter(pred, self)
    }
    #[inline]
    fn filter_map<F, T>(self, filter: F) -> convert::FilterMap<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Option<T>,
        T: 'static + Clone,
    {
        convert::filter_map(filter, self)
    }

    #[inline]
    fn and_then<F, T>(self, op: F) -> convert::AndThen<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> Result<T, E>,
        T: 'static + Clone,
    {
        convert::and_then(op, self)
    }
    #[inline]
    fn then_some<T>(self, value: T) -> convert::ThenSome<'i, U, E, Self, T>
    where
        Self: Sized,
        T: Clone,
    {
        convert::then_some(value, self)
    }

    #[inline]
    fn complex<Q>(self, then: Q) -> convert::Complex<'i, U, E, Self, Q>
    where
        Self: Sized,
        Q: Pattern<'i, U, E>,
    {
        convert::complex(self, then)
    }

    #[inline]
    fn map<F, T>(self, op: F) -> convert::Map<'i, U, E, Self, F, T>
    where
        Self: Sized,
        F: Fn(Self::Captured) -> T,
    {
        convert::map(op, self)
    }
    #[inline]
    fn map_err<F, E2>(self, op: F) -> convert::MapErr<'i, U, E, Self, F, E2>
    where
        Self: Sized,
        F: Fn(E) -> E2,
        E2: Situation,
    {
        convert::map_err(op, self)
    }

    #[inline]
    fn expect(self, msg: &'static str) -> convert::Expect<'i, U, E, Self>
    where
        Self: Sized,
    {
        convert::expect(msg, self)
    }
    #[inline]
    fn unwrap(self) -> convert::Unwrap<'i, U, E, Self>
    where
        Self: Sized,
    {
        convert::unwrap(self)
    }

    #[inline]
    fn unwrap_or(self, default: Self::Captured) -> convert::UnwrapOr<'i, U, E, Self>
    where
        Self: Sized,
        Self::Captured: Clone,
    {
        convert::unwrap_or(default, self)
    }
    #[inline]
    fn unwrap_or_else<F>(self, f: F) -> convert::UnwrapOrElse<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn() -> Self::Captured,
    {
        convert::unwrap_or_else(f, self)
    }
    #[inline]
    fn unwrap_or_default(self) -> convert::UnwrapOrDefault<'i, U, E, Self>
    where
        Self: Sized,
        Self::Captured: Default,
    {
        convert::unwrap_or_default(self)
    }

    //------------------------------------------------------------------------------

    #[inline]
    fn parallel(self) -> modifier::Parallel<'i, U, E, Self>
    where
        Self: Sized,
    {
        modifier::parallel(self)
    }

    #[inline]
    fn trace<I>(self, info: I) -> modifier::Trace<'i, U, E, Self, I>
    where
        Self: Sized,
        I: core::fmt::Display,
    {
        modifier::trace(info, self)
    }

    #[inline]
    fn desc(self, desc: E::Description) -> modifier::Describe<'i, U, E, Self>
    where
        Self: Sized,
        E::Description: Clone,
    {
        modifier::desc(desc, self)
    }
    #[inline]
    fn desc_with<F>(self, f: F) -> modifier::DescribeWith<'i, U, E, Self, F>
    where
        Self: Sized,
        F: Fn(&E) -> E::Description,
    {
        modifier::desc_with(f, self)
    }

    #[inline]
    fn void(self) -> modifier::Void<'i, U, E, Self>
    where
        Self: Sized,
    {
        modifier::void(self)
    }
}

//==================================================================================================

impl<'i, U, E> Pattern<'i, U, E> for &'i U
where
    U: ?Sized + Slice + 'i,
    E: Situation,
{
    type Captured = &'i U;
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}

    #[inline]
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

    #[inline]
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.split_at(self.len()).0
    }
}

impl<'i, U, E, P> Pattern<'i, U, E> for [P; 1]
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Predicate<U::Item>,
{
    type Captured = U::Item;
    type Internal = ();

    #[inline]
    fn init(&self) -> Self::Internal {}

    #[inline]
    fn advance(&self, slice: &U, _ntry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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
    fn extract(&self, slice: &'i U, _ntry: Self::Internal) -> Self::Captured {
        slice.first().unwrap()
    }
}

//==================================================================================================

pub trait IndexedPattern<'i, U, E>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
{
    type Captured;
    type Internal;

    fn init_ixs(&self) -> Self::Internal;
    fn advance_ixs(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;
    fn extract_ixs(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;

    #[inline]
    fn parse_indexed(&self, slice: &mut dyn AdvanceSlice<'i, U>) -> Result<Self::Captured, E> {
        let mut state = self.init_ixs();
        match self.advance_ixs(slice.rest(), &mut state, true) {
            Ok(len) => Ok(self.extract_ixs(slice.bump(len), state)),
            Err(e) if e.is_unfulfilled() => panic!("implementation: pull after EOF"),
            Err(e) => Err(e.backtrack(slice.consumed())),
        }
    }
    #[inline]
    fn fullmatch_indexed(&self, slice: &'i U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse_indexed(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_halt_at(slice.len() - n),
        }
    }
}

macro_rules! impl_indexed_pattern_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Indexedable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = ($((usize, $GenN::Captured),)+);
            type Internal = ([<Check $Len>], ($((usize, $GenN::Internal),)+));

            #[inline]
            fn init_ixs(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance_ixs(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (off, state) = &mut states.$IdxN;
                        if likely(*off == 0) {
                            *off = offset;
                        }

                        match self.$IdxN.advance(slice.split_at(*off).1, state, eof) {
                            Ok(len) => offset = *off + len,
                            Err(e) => return e.raise_backtrack(*off),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract_ixs(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = ($ValN.0, self.$IdxN.extract(slice.split_at($ValN.0).1, $ValN.1));
                )+
                ($($ValN,)+)
            }
        }
    } };
}

// __generate_codes! { impl_indexed_pattern_for_tuple ( P ~ val ) }

//==================================================================================================

pub trait SpannedPattern<'i, U, E>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
{
    type Captured;
    type Internal: 'static + Clone;

    fn init_sps(&self) -> Self::Internal;
    fn advance_sps(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E>;
    fn extract_sps(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured;

    #[inline]
    fn parse_spanned(&self, slice: &mut dyn AdvanceSlice<'i, U>) -> Result<Self::Captured, E> {
        let mut state = self.init_sps();
        match self.advance_sps(slice.rest(), &mut state, true) {
            Ok(len) => Ok(self.extract_sps(slice.bump(len), state)),
            Err(e) if e.is_unfulfilled() => panic!("implementation: pull after EOF"),
            Err(e) => Err(e.backtrack(slice.consumed())),
        }
    }
    #[inline]
    fn fullmatch_spanned(&self, slice: &'i U) -> Result<Self::Captured, E> {
        let mut sli = slice;
        let cap = self.parse_spanned(&mut sli)?;
        match sli.len() {
            0 => Ok(cap),
            n => E::raise_halt_at(slice.len() - n),
        }
    }
}

macro_rules! impl_spanned_pattern_for_tuple {
    ( $Len:literal, $($OrdN:literal ~ ($GenN:ident ~ $ValN:ident) ~ $_gen:ident ~ $_con:ident ~ $IdxN:tt)+ ) => { paste::paste! {
        impl<'i, U, E, $($GenN: Pattern<'i, U, E>),+> Spannedable<'i, U, E> for ($($GenN,)+)
        where
            U: ?Sized + Slice + 'i,
            E: Situation,
        {
            type Captured = ($((Range<usize>, $GenN::Captured),)+);
            type Internal = ([<Check $Len>], ($((Range<usize>, $GenN::Internal),)+));

            #[inline]
            fn init_sps(&self) -> Self::Internal {
                ([<Check $Len>]::Point1, ($((0..0, self.$IdxN.init()),)+))
            }

            #[inline]
            fn advance_sps(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                use [<Check $Len>]::*;
                let (checkpoint, states) = entry;
                let mut offset = 0usize;

                __resume_advance! { *checkpoint ; $(
                    [<Point $OrdN>] => {
                        *checkpoint = [<Point $OrdN>];
                    } {
                        let (span, state) = &mut states.$IdxN;
                        if likely(span.start == 0) {
                            span.start = offset;
                        }

                        match self.$IdxN.advance(slice.split_at(span.start).1, state, eof) {
                            Ok(len) => { offset = span.start + len ; span.end = offset }
                            Err(e) => return e.raise_backtrack(span.start),
                        }
                    }
                )+ }

                Ok(offset)
            }

            #[inline]
            fn extract_sps(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                $(
                    let $ValN = entry.1.$IdxN;
                    let $ValN = ($ValN.0.clone(), self.$IdxN.extract(slice.split_at($ValN.0.start).1, $ValN.1));
                )+
                ($($ValN,)+)
            }
        }
    } };
}

// __generate_codes! { impl_spanned_pattern_for_tuple ( P ~ val ) }

//==================================================================================================

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn slice() {
        let pat = impls::opaque_simple("");
        assert!(pat.fullmatch("").is_ok());
        assert_eq!(pat.fullmatch("?").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("??").unwrap_err().offset(), 0);

        let pat = impls::opaque_simple("A");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("A").unwrap(), "A");
        assert_eq!(pat.fullmatch("AA").unwrap_err().offset(), 1);

        let pat = impls::opaque_simple("AB");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("AB").unwrap(), "AB");
        assert_eq!(pat.fullmatch("ABCD").unwrap_err().offset(), 2);

        let pat = impls::opaque_simple("ABCD");
        assert_eq!(pat.fullmatch("").unwrap_err().offset(), 0);
        assert_eq!(pat.fullmatch("AB").unwrap_err().offset(), 2);
        assert_eq!(pat.fullmatch("ABCD").unwrap(), "ABCD");
    }
}
