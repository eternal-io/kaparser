use crate::{common::*, slice::*};
use core::{convert::Infallible, ops::Range};

pub trait Input<'src, 'tmp> {
    type Token;
    type TokenMaybe<'once>: MaybeRef<'once, Self::Token>
    where
        'src: 'once,
        'once: 'tmp;

    type Error;
    type Cursor: Clone;

    fn begin(&self) -> Self::Cursor;

    /// # Safety
    ///
    /// If `'tmp` doesn't outlives `'src`, the returned value must be dropped before:
    ///
    /// - Calling any other method of this trait (or super-trait), or
    /// - Ending the mutable borrow of the input.
    ///
    /// Violating this contract may cause undefined behavior.
    unsafe fn next_maybe_ref<'once>(
        &mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'once>>, Self::Error>
    where
        'src: 'once,
        'once: 'tmp;

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize>;

    fn offset(&self, cursor: Self::Cursor) -> usize;
}

pub trait InputOwnableToken<'src, 'tmp>: Input<'src, 'tmp> {
    fn next(&mut self, cursor: Self::Cursor) -> Result<Option<Self::Token>, Self::Error>;
}

pub trait InputBorrowableToken<'src, 'tmp>: Input<'src, 'tmp> {
    fn next_ref(&mut self, cursor: Self::Cursor) -> Result<Option<&'src Self::Token>, Self::Error>;
}

pub trait InputSlice<'src, 'tmp>: Input<'src, 'tmp> {
    type Slice: ?Sized + Slice<'tmp, Item = Self::Token> + 'tmp;

    /// # Safety
    ///
    /// If `'tmp` doesn't outlives `'src`, the returned slice must be dropped before:
    ///
    /// - Calling any other method of this trait (or super-trait), or
    /// - Ending the mutable borrow of the input.
    ///
    /// Violating this contract may cause undefined behavior.
    unsafe fn acquire_slice<'once>(&mut self, cursor: Self::Cursor) -> Result<(&'once Self::Slice, bool), Self::Error>
    where
        'src: 'once,
        'once: 'tmp;

    /// # Safety
    ///
    /// If `'tmp` doesn't outlives `'src`, the returned slice must be dropped before:
    ///
    /// - Calling any other method of this trait (or super-trait), or
    /// - Ending the mutable borrow of the input.
    ///
    /// Violating this contract may cause undefined behavior.
    unsafe fn discard_slice<'once>(&mut self, cursor: &mut Self::Cursor, length: usize) -> &'once Self::Slice
    where
        'src: 'once,
        'once: 'tmp;
}

pub trait InputThinSlice<'src, 'tmp>: InputSlice<'src, 'tmp, Token = u8> {}

#[cfg(feature = "alloc")]
pub trait InputBoxableSlice<'src, 'tmp>: InputSlice<'src, 'tmp>
where
    Self::Slice: BoxableSlice<'tmp, Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

impl<'src> Input<'src, 'src> for &'src str {
    type Token = char;
    type TokenMaybe<'once>
        = char
    where
        'src: 'once,
        'once: 'src;
    type Error = Infallible;
    type Cursor = usize;

    fn begin(&self) -> Self::Cursor {
        todo!()
    }

    unsafe fn next_maybe_ref<'once>(
        &mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'once>>, Self::Error>
    where
        'src: 'once,
        'once: 'src,
    {
        Ok(self[*cursor..].chars().next())
    }

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        todo!()
    }

    fn offset(&self, cursor: Self::Cursor) -> usize {
        todo!()
    }
}

impl<'src> InputSlice<'src, 'src> for &'src str {
    type Slice = str;

    unsafe fn acquire_slice<'once>(&mut self, cursor: Self::Cursor) -> Result<(&'once Self::Slice, bool), Self::Error>
    where
        'src: 'once,
        'once: 'src,
    {
        todo!()
    }

    unsafe fn discard_slice<'once>(&mut self, cursor: &mut Self::Cursor, length: usize) -> &'once Self::Slice
    where
        'src: 'once,
        'once: 'src,
    {
        *self
    }
}

//------------------------------------------------------------------------------

use alloc::string::String;

impl<'src, 'tmp> Input<'src, 'tmp> for String {
    type Token = char;
    type TokenMaybe<'once>
        = char
    where
        'src: 'once,
        'once: 'tmp;
    type Error = Infallible;
    type Cursor = usize;

    fn begin(&self) -> Self::Cursor {
        todo!()
    }

    unsafe fn next_maybe_ref<'once>(
        &mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'once>>, Self::Error>
    where
        'src: 'once,
        'once: 'tmp,
    {
        Ok(self[*cursor..].chars().next())
    }

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        todo!()
    }

    fn offset(&self, cursor: Self::Cursor) -> usize {
        todo!()
    }
}

impl<'src, 'tmp> InputSlice<'src, 'tmp> for String {
    type Slice = str;

    unsafe fn acquire_slice<'once>(&mut self, cursor: Self::Cursor) -> Result<(&'once Self::Slice, bool), Self::Error>
    where
        'src: 'once,
        'once: 'tmp,
    {
        todo!()
    }

    unsafe fn discard_slice<'once>(&mut self, cursor: &mut Self::Cursor, length: usize) -> &'once Self::Slice
    where
        'src: 'once,
        'once: 'tmp,
    {
        unsafe { core::mem::transmute(self.as_str()) }
    }
}
