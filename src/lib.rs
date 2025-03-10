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
pub mod anything {
    //! Re-exports all items in kaparser.
    //!
    //! This module can help define `fn combinator`s with type annotations...
    //! if you *really* want.
    //!
    //! Before doing so, please check [`pattern::def`] and [`pattern::bin`],
    //! maybe the patterns you want are already predefined.
    //! If they're not there and you're sure these patterns are commonly used,
    //! feel free to open an issue.
    //!
    //! [`pattern::def`]: crate::pattern::def
    //! [`pattern::bin`]: crate::pattern::bin
    #[doc(hidden)]
    pub use crate::{
        combine::*,
        common::{alts::*, *},
        error::*,
        pattern::{impls::*, *},
        predicate::*,
        provider::*,
    };
    #[doc(hidden)]
    pub use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
}
