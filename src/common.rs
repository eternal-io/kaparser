use crate::predicate::*;
use core::fmt;

//------------------------------------------------------------------------------

pub trait Describe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<'a> fmt::Display for &'a dyn Describe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).fmt(f)
    }
}

//------------------------------------------------------------------------------

pub trait RefVal<'tmp, T: 'tmp> {
    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool;

    fn as_ref(&self) -> &T;

    fn cloned(&self) -> T
    where
        T: Clone;
}

impl<'tmp, T: 'tmp> RefVal<'tmp, T> for &'tmp T {
    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool {
        pred.predicate(self)
    }

    fn as_ref(&self) -> &T {
        self
    }

    fn cloned(&self) -> T
    where
        T: Clone,
    {
        (*self).clone()
    }
}

impl<'tmp, T: 'tmp> RefVal<'tmp, T> for T {
    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool {
        pred.predicate(self)
    }

    fn as_ref(&self) -> &T {
        self
    }

    fn cloned(&self) -> T
    where
        T: Clone,
    {
        (*self).clone()
    }
}

//------------------------------------------------------------------------------
