#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use core::{error::Error, fmt, mem, num::NonZeroU16, ops::Range, ptr::copy_nonoverlapping, str::from_utf8_unchecked};

#[macro_use]
pub mod common;

mod combine;
mod predicate;
mod proceed;

pub use crate::{combine::*, predicate::*, proceed::*};

pub type TheResult<T> = Result<T, Box<dyn Error>>;

pub fn main() {
    resume_proceed! {
        'l: check => {
            'foo: p1 => {
                do_stuff1();
            }
            'bar: p2 => {
                do_stuff2();
            }
            'baz: p3 => {
                do_stuff3();
            }
        }
    }
}
