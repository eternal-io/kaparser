#![no_std]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod macros;

#[macro_use]
pub mod common;
pub mod combine;
pub mod parser;
pub mod pattern;
pub mod predicate;
pub mod prelude;
pub mod anything {
    //! This module re-exports all items in kaparser.
    //!
    //! It can help define `const COMBINATOR`s with type annotations...
    //! if you *really* want.
    //!
    //! Before doing so, please check [`combine::def`] and [`combine::bin`],
    //! maybe the patterns you want are already defined.
    //! If they're not there and you're sure these patterns are commonly used,
    //! feel free to open an issue.
    //!
    //! [`combine::def`]: crate::combine::def
    //! [`combine::bin`]: crate::combine::bin
    #[doc(hidden)]
    pub use crate::{combine::*, common::*, parser::*, pattern::*, predicate::*};
    #[doc(hidden)]
    pub use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
}
