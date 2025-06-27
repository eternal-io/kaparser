use crate::predicate::*;
use core::{
    fmt,
    ops::{Deref, DerefMut},
};

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

pub enum MaybeRef<'a, T> {
    Ref(&'a T),
    Val(T),
}

impl<'a, T> MaybeRef<'a, T> {
    pub fn share(&'a self) -> Self {
        match self {
            MaybeRef::Ref(v) => Self::Ref(v),
            MaybeRef::Val(v) => Self::Ref(v),
        }
    }
}

impl<'a, T> AsRef<T> for MaybeRef<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            MaybeRef::Ref(v) => v,
            MaybeRef::Val(v) => v,
        }
    }
}

impl<'a, T> Deref for MaybeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeRef::Ref(v) => v,
            MaybeRef::Val(v) => v,
        }
    }
}

impl<'a, T> From<T> for MaybeRef<'a, T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

impl<'a, T> From<&'a T> for MaybeRef<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Ref(value)
    }
}

//------------------------------------------------------------------------------

pub enum MaybeMut<'a, T> {
    Mut(&'a mut T),
    Val(T),
}

impl<'a, T> MaybeMut<'a, T> {
    pub fn share(&'a mut self) -> Self {
        match self {
            MaybeMut::Mut(v) => Self::Mut(v),
            MaybeMut::Val(v) => Self::Mut(v),
        }
    }
}

impl<'a, T> AsRef<T> for MaybeMut<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            MaybeMut::Mut(v) => v,
            MaybeMut::Val(v) => v,
        }
    }
}

impl<'a, T> AsMut<T> for MaybeMut<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        match self {
            MaybeMut::Mut(v) => v,
            MaybeMut::Val(v) => v,
        }
    }
}

impl<'a, T> Deref for MaybeMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeMut::Mut(v) => v,
            MaybeMut::Val(v) => v,
        }
    }
}

impl<'a, T> DerefMut for MaybeMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeMut::Mut(v) => v,
            MaybeMut::Val(v) => v,
        }
    }
}

impl<'a, T> From<T> for MaybeMut<'a, T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

impl<'a, T> From<&'a mut T> for MaybeMut<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Self::Mut(value)
    }
}

//------------------------------------------------------------------------------
