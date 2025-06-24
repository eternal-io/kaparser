use crate::predicate::*;

pub trait RefVal<'tmp, T: 'tmp> {
    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool;

    fn as_ref(&self) -> &'tmp T;

    fn cloned(&self) -> T
    where
        T: Clone;
}

impl<'tmp, T: 'tmp> RefVal<'tmp, T> for &'tmp T {
    fn predicate<P: Predicate<T>>(&self, pred: &P) -> bool {
        pred.predicate(self)
    }

    fn as_ref(&self) -> &'tmp T {
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

    fn as_ref(&self) -> &'tmp T {
        unimplemented!()
    }

    fn cloned(&self) -> T
    where
        T: Clone,
    {
        (*self).clone()
    }
}

//------------------------------------------------------------------------------
