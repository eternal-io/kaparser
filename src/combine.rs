#![doc = include_str!("../docs/combinators.md")]
use crate::{common::*, error::*, pattern::*, predicate::*};
use core::marker::PhantomData;

pub mod alt;
pub mod com;
pub mod disp;
pub mod perm;
pub mod seq;

pub mod lens;
pub mod repeat;
pub mod skim;
pub mod take;

pub mod opt;
pub mod peek;
pub mod winged;

pub mod control;
pub mod convert;
pub mod modifier;

pub mod not;
