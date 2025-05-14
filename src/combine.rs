#![doc = include_str!("../docs/combinators.md")]
use crate::{common::*, error::*, pattern::*, predicate::*};
use core::marker::PhantomData;

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
    pub mod permute;
    pub mod dispatch;

    pub mod take;
    pub mod lens;
    pub mod repeat;
    pub mod skim;

    pub mod opt;
    pub mod peek;
    pub mod winged;
    pub mod control;
    pub mod convert;
    pub mod modifier;

    pub mod not;
}
