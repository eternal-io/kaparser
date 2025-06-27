#![no_std]

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod marker;

#[macro_use]
pub mod common;
pub mod combinator;
pub mod error;
pub mod extra;
pub mod input;
pub mod parser;
pub mod pattern;
pub mod predicate;
pub mod primitive;
pub mod quattrn;
pub mod slice;
