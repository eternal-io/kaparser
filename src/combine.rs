use crate::{common::*, precede::*, predicate::*};
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
    pub mod cut;
    pub mod not;
    pub mod opt;
    pub mod reiter;
    pub mod repeat;
    pub mod seq;
    pub mod skim;
    pub mod take;
    pub mod with;
}

import! {
    pub mod bin;
    pub mod def;
}
