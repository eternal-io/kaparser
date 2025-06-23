#![no_std]

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod common;
pub mod input;
pub mod pattern;
pub mod predicate;
pub mod slice;

//------------------------------------------------------------------------------

pub trait Slice {
    fn identity(&self) -> &Self {
        self
    }
}

pub trait InputSlice<'src, 'tmp> {
    type View: ?Sized + Slice + 'tmp;

    /// # Safety
    ///
    /// If `'tmp` doesn't outlives `'src`, the returned slice must be dropped before:
    ///
    /// - Calling any other method of this trait (and super-trait), or
    /// - Ending the mutable borrow of the input.
    ///
    /// Violating this contract may cause undefined behavior.
    unsafe fn get_slice<'once>(&mut self) -> &'once Self::View
    where
        'src: 'once,
        'once: 'tmp;
}

impl Slice for str {}

impl<'src> InputSlice<'src, 'src> for &'src str {
    type View = str;

    unsafe fn get_slice<'once>(&mut self) -> &'once Self::View
    where
        'src: 'once,
        'once: 'src,
    {
        *self
    }
}

use alloc::string::String;

impl<'src, 'tmp> InputSlice<'src, 'tmp> for String {
    type View = str;

    unsafe fn get_slice<'once>(&mut self) -> &'once Self::View
    where
        'src: 'once,
        'once: 'tmp,
    {
        unsafe { core::mem::transmute(self.as_str()) }
    }
}

#[test]
#[ignore = "undefined behavior"]
fn foo() {
    let mut s = String::new();
    let a = unsafe { s.get_slice() };
    let b = unsafe { s.get_slice() };
    println!("{}, {}", a, b); // UB!!
}
