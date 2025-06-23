use crate::predicate::*;

pub trait RefVal<'tmp, T: 'tmp> {
    fn identity(self) -> Self;

    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool;
}

impl<'tmp, T: 'tmp> RefVal<'tmp, T> for &'tmp T {
    fn identity(self) -> Self {
        self
    }

    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool {
        pred.predicate(self)
    }
}

impl<'tmp, T: 'tmp> RefVal<'tmp, T> for T {
    fn identity(self) -> Self {
        self
    }

    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool {
        pred.predicate(self)
    }
}

//------------------------------------------------------------------------------
