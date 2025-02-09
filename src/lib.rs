#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use core::{
    error::Error,
    fmt, mem,
    num::NonZeroU16,
    ops::Range,
    ptr::{addr_of_mut, copy_nonoverlapping},
    str::from_utf8_unchecked,
};

#[doc(hidden)]
pub use paste::paste;

pub mod common;

mod helper;
mod precede;
mod predicate;

pub use crate::{helper::*, precede::*, predicate::*};

pub type TheResult<T> = Result<T, Box<dyn Error>>;
