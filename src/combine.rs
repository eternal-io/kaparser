use crate::{common::*, pattern::*, predicate::*};
use ::core::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
};

macro_rules! import {
    ($($vis:vis mod $name:ident;)*) => { $(
        $vis mod $name;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use $name::*;
    )* };
}

import! {
    pub mod alt;
    pub mod com;
    pub mod convert;
    pub mod cut;
    pub mod lens;
    pub mod not;
    pub mod opt;
    pub mod reiter;
    pub mod repeat;
    pub mod seq;
    pub mod skim;
    pub mod take;
}

import! {
    pub mod bin;
    pub mod def;
}
