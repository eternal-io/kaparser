#![no_std]
#![deny(unsafe_code)]

#[macro_use]
mod macros;

#[macro_use]
pub mod common;
pub mod error;
pub mod line_col;
pub mod pattern;
pub mod predicate;
pub mod prelude;

#[rustfmt::skip]
#[doc = include_str!("../docs/combinators.md")]
pub mod combine {
    #[cfg(test)]
    use crate::tester::*;
    use crate::{common::*, prelude::*};
    use core::marker::PhantomData;

    pub mod alt;
    pub mod com;
    pub mod seq;
    pub mod dispatch;

    pub mod lens;
    pub mod take;
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

#[cfg(test)]
mod tester;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;
