#![no_std]

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod marker;

pub mod common;
pub mod input;
pub mod pattern;
pub mod predicate;
pub mod slice;
