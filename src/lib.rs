#![no_std]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

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
pub mod provider;
