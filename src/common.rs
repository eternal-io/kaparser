use crate::{error::Error, predicate::*};
use core::{
    fmt,
    ops::{Deref, DerefMut},
};

pub struct PResult<T, E> {
    pub(crate) value: Option<T>,
    pub(crate) error: Option<E>,
}

impl<T, E> PResult<T, E> {
    pub fn has_output(&self) -> bool {
        self.value.is_some()
    }
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn output(&self) -> Option<&T> {
        self.value.as_ref()
    }
    pub fn error(&self) -> Option<&E> {
        self.error.as_ref()
    }

    pub fn into_output(self) -> Option<T> {
        self.value
    }
    pub fn into_error(self) -> Option<E> {
        self.error
    }

    pub fn into_output_error(self) -> (Option<T>, Option<E>) {
        (self.value, self.error)
    }

    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        if let Some(e) = self.error {
            Err(e)
        } else if let Some(val) = self.value {
            Ok(val)
        } else {
            unreachable!()
        }
    }

    #[track_caller]
    pub fn unwrap(self) -> T {
        self.error.unwrap();
        self.value.unwrap()
    }

    #[inline]
    pub(crate) fn verify_map<F, U>(self, f: F) -> PResult<U, E>
    where
        E: Error,
        F: FnOnce(T) -> (U, Option<E>),
    {
        let PResult { value, error: err1 } = self;

        if let Some(val) = value {
            let (out, err2) = f(val);

            PResult {
                value: Some(out),
                error: match (err1, err2) {
                    (None, None) => None,
                    (Some(e1), None) => Some(e1),
                    (None, Some(e2)) => Some(e2),
                    (Some(e1), Some(e2)) => Some(e1.merge(e2)),
                },
            }
        } else {
            PResult {
                value: None,
                error: err1,
            }
        }
    }

    #[inline]
    pub(crate) fn raise_or_map<F, U>(self, f: F) -> PResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self.into_result() {
            Ok(val) => PResult::submit(f(val)),
            Err(e) => PResult::raise(e),
        }
    }

    #[inline]
    pub(crate) fn map<F, U>(self, f: F) -> PResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        PResult {
            value: self.value.map(f),
            error: self.error,
        }
    }

    #[inline]
    pub(crate) fn map_err<F, E2>(self, f: F) -> PResult<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        PResult {
            value: self.value,
            error: self.error.map(f),
        }
    }

    #[inline]
    pub(crate) fn submit(value: T) -> PResult<T, E> {
        PResult {
            value: Some(value),
            error: None,
        }
    }

    #[inline]
    pub(crate) fn raise(error: E) -> PResult<T, E> {
        PResult {
            value: None,
            error: Some(error),
        }
    }
}

//------------------------------------------------------------------------------

pub trait Describe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for &dyn Describe {
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

macro_rules! dyn_coerce {
    ( $expr:expr $(=> $trait:path)* ) => {
        dyn_coerce! { @ $expr $(=> $trait)* }
    };

    ( @ $expr:expr => $trait:path $(=> $traits:path)* ) => {
        dyn_coerce! { @ &$expr as &dyn $trait $(=> $traits)* }
    };

    ( @ $expr:expr ) => { $expr };
}

//------------------------------------------------------------------------------

macro_rules! __forward_check {
    ( $p:ident ) => {
        fn __check(
            &self,
            input: &mut I,
            start: I::Cursor,
            state: MaybeMut<Ext::State>,
            ctx: MaybeRef<Ext::Context>,
            _: private::Token,
        ) -> PResult<I::Cursor, Ext::Error> {
            self.$p.__check(input, start, state, ctx, private::Token)
        }
    };
}

//------------------------------------------------------------------------------

/// `Lens1X` means `LenX - 1`. Always `N < K < M`.
/// `Gen` means "Generic". "Con" means "Converge".
macro_rules! __generate_codes {
    ( $callback:ident $(($($custom:ident) ~ +))? ) => { paste::paste! {
        __generate_codes! {
          @ $callback ;
            0  ~ 1  $(~ ($([< $custom 1  >]) ~ +))? ~ A ~ A ~ 0
            1  ~ 2  $(~ ($([< $custom 2  >]) ~ +))? ~ B ~ A ~ 1
            2  ~ 3  $(~ ($([< $custom 3  >]) ~ +))? ~ C ~ A ~ 2
            3  ~ 4  $(~ ($([< $custom 4  >]) ~ +))? ~ D ~ A ~ 3
            4  ~ 5  $(~ ($([< $custom 5  >]) ~ +))? ~ E ~ A ~ 4
            5  ~ 6  $(~ ($([< $custom 6  >]) ~ +))? ~ F ~ A ~ 5
            6  ~ 7  $(~ ($([< $custom 7  >]) ~ +))? ~ G ~ A ~ 6
            7  ~ 8  $(~ ($([< $custom 8  >]) ~ +))? ~ H ~ A ~ 7
            8  ~ 9  $(~ ($([< $custom 9  >]) ~ +))? ~ I ~ A ~ 8
            9  ~ 10 $(~ ($([< $custom 10 >]) ~ +))? ~ J ~ A ~ 9
            10 ~ 11 $(~ ($([< $custom 11 >]) ~ +))? ~ K ~ A ~ 10
            11 ~ 12 $(~ ($([< $custom 12 >]) ~ +))? ~ L ~ A ~ 11
            12 ~ 13 $(~ ($([< $custom 13 >]) ~ +))? ~ M ~ A ~ 12
            13 ~ 14 $(~ ($([< $custom 14 >]) ~ +))? ~ N ~ A ~ 13
            14 ~ 15 $(~ ($([< $custom 15 >]) ~ +))? ~ O ~ A ~ 14
            15 ~ 16 $(~ ($([< $custom 16 >]) ~ +))? ~ P ~ A ~ 15
            16 ~ 17 $(~ ($([< $custom 17 >]) ~ +))? ~ Q ~ A ~ 16
            17 ~ 18 $(~ ($([< $custom 18 >]) ~ +))? ~ R ~ A ~ 17
            18 ~ 19 $(~ ($([< $custom 19 >]) ~ +))? ~ S ~ A ~ 18
            19 ~ 20 $(~ ($([< $custom 20 >]) ~ +))? ~ T ~ A ~ 19
            20 ~ 21 $(~ ($([< $custom 21 >]) ~ +))? ~ U ~ A ~ 20
            21 ~ 22 $(~ ($([< $custom 22 >]) ~ +))? ~ V ~ A ~ 21
            22 ~ 23 $(~ ($([< $custom 23 >]) ~ +))? ~ W ~ A ~ 22
        }
    } };

    ( @ $callback:ident ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        __generate_codes! {
          @ $callback ;
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        $callback!( $Lens1K, $($OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+ );
        __generate_codes! {
          @ $callback ;
          $($Lens1N ~ $OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
    ) => {};
}
