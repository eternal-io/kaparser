#![doc = include_str!("../docs/combinators.md")]
use crate::{common::*, pattern::*, predicate::*};
use core::{marker::PhantomData, mem::MaybeUninit};

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
    pub mod seq;

    pub mod take;
    pub mod lens;
    pub mod repeat;

    pub mod skim;

    pub mod opt;
    pub mod ctrl;

    pub mod not;
}

import! {
    pub mod bin;
    pub mod def;
}
