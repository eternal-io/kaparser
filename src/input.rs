use crate::{common::*, marker, slice::*};
use core::{convert::Infallible, ops::Range};

pub trait Input<'src>: 'src {
    type _Mark;

    type Token: 'src;
    type TokenMaybe<'tmp>: RefVal<'tmp, Self::Token>
    where
        'src: 'tmp;

    type Error;
    type Cursor: Clone;

    fn begin(&self) -> Self::Cursor;

    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'tmp>>, Self::Error>
    where
        'src: 'tmp;

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize>;

    fn offset(&self, cursor: Self::Cursor) -> usize;
}

pub trait InputOwnableToken<'src>: Input<'src> {
    fn next(&mut self, cursor: Self::Cursor) -> Result<Option<Self::Token>, Self::Error>;
}

pub trait InputBorrowableToken<'src>: Input<'src> {
    fn next_ref(&mut self, cursor: Self::Cursor) -> Result<Option<&'src Self::Token>, Self::Error>;
}

pub trait InputSlice<'src>: Input<'src> {
    type Slice: ?Sized + Slice<'src, Item = Self::Token> + 'src;

    fn acquire_slice<'tmp>(&'tmp mut self, cursor: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp;

    fn discard_slice<'tmp>(&'tmp mut self, cursor: &mut Self::Cursor, length: usize) -> &'tmp Self::Slice
    where
        'src: 'tmp;
}

pub trait InputThinSlice<'src>: InputSlice<'src, Token = u8> {}

#[cfg(feature = "alloc")]
pub trait InputBoxableSlice<'src>: InputSlice<'src>
where
    Self::Slice: BoxableSlice<'src, Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

impl<'src> Input<'src> for &'src str {
    type _Mark = marker::StaticInput;
    type Token = char;
    type TokenMaybe<'tmp>
        = char
    where
        'src: 'tmp;
    type Error = Infallible;
    type Cursor = usize;

    fn begin(&self) -> Self::Cursor {
        todo!()
    }

    fn next_maybe_ref<'tmp>(&mut self, cursor: &mut Self::Cursor) -> Result<Option<Self::TokenMaybe<'tmp>>, Self::Error>
    where
        'src: 'tmp,
    {
        todo!()
    }

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        todo!()
    }

    fn offset(&self, cursor: Self::Cursor) -> usize {
        todo!()
    }
}

impl<'src> InputSlice<'src> for &'src str {
    type Slice = str;

    fn acquire_slice<'tmp>(&'tmp mut self, cursor: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp,
    {
        todo!()
    }

    fn discard_slice<'tmp>(&'tmp mut self, cursor: &mut Self::Cursor, length: usize) -> &'tmp Self::Slice
    where
        'src: 'tmp,
    {
        self
    }
}

//------------------------------------------------------------------------------

use alloc::string::String;

impl<'src> Input<'src> for String {
    type _Mark = marker::DynamicInput;

    type Token = char;
    type TokenMaybe<'tmp>
        = char
    where
        'src: 'tmp;

    type Error = Infallible;
    type Cursor = usize;

    fn begin(&self) -> Self::Cursor {
        todo!()
    }

    fn next_maybe_ref<'tmp>(&mut self, cursor: &mut Self::Cursor) -> Result<Option<Self::TokenMaybe<'tmp>>, Self::Error>
    where
        'src: 'tmp,
    {
        todo!()
    }

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        todo!()
    }

    fn offset(&self, cursor: Self::Cursor) -> usize {
        todo!()
    }
}

impl<'src> InputSlice<'src> for String {
    type Slice = str;

    fn acquire_slice<'tmp>(&'tmp mut self, cursor: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp,
    {
        todo!()
    }

    fn discard_slice<'tmp>(&'tmp mut self, cursor: &mut Self::Cursor, length: usize) -> &'tmp Self::Slice
    where
        'src: 'tmp,
    {
        self.as_str()
    }
}
