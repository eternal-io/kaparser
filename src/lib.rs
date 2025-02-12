#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports, unused_variables)]
#![deny(unsafe_code)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use core::{error::Error, fmt, mem, num::NonZeroU16, ops::Range, ptr::copy_nonoverlapping, str::from_utf8_unchecked};

#[macro_use]
mod macros;

#[macro_use]
pub mod common;
pub mod combine;
pub mod parser;
pub mod predicate;
pub mod prelude;
pub mod proceed;

pub type TheResult<T> = Result<T, Box<dyn Error>>;
