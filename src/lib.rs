#![no_std]
#![deny(unsafe_code)]
// temporary !!!
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
mod macros;

#[macro_use]
pub mod common;
pub mod combine;
pub mod error;
pub mod line_col;
pub mod pattern;
pub mod predicate;
pub mod prelude;

#[cfg(test)]
mod tester;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;
