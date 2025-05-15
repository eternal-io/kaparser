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
#[allow(unused_variables)]
#[doc = include_str!("../docs/combinators.md")]
pub mod combine {
    use crate::{common::*, prelude::*};
    use core::marker::PhantomData;

    pub mod alt;
    pub mod com;
    pub mod seq;
    pub mod dispatch;
    pub mod permute;

    pub mod lens;
    pub mod repeat;
    pub mod skim;
    pub mod take;

    pub mod opt;
    pub mod peek;
    pub mod behind;
    pub mod winged;

    pub mod control;
    pub mod convert;
    pub mod modifier;

    pub mod not;
}
