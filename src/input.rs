use crate::{common::*, slice::*};
use core::{convert::Infallible, ops::Range};

pub trait Input<'src>: 'src {
    type Token: 'src;
    type TokenMaybe<'tmp>: MaybeRef<'tmp, Self::Token>
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

pub trait InputOwnableToken<'src>: Input<'src>
where
    Self::Token: Sized,
{
    fn next(&mut self, cursor: Self::Cursor) -> Result<Option<Self::Token>, Self::Error>;
}

pub trait InputBorrowableToken<'src>: Input<'src> {
    fn next_ref(&mut self, cursor: Self::Cursor) -> Result<Option<&'src Self::Token>, Self::Error>;
}

pub trait InputSlice<'src>: Input<'src> {
    type Slice<'tmp>: Slice<Item = Self::Token>;

    fn acquire_slice<'tmp>(&mut self, cursor: Self::Cursor) -> Result<(Self::Slice<'tmp>, bool), Self::Error>;

    fn discard_slice<'tmp>(&mut self, cursor: &mut Self::Cursor, length: usize) -> Self::Slice<'tmp>;
}

pub trait InputThinSlice<'src>: InputSlice<'src, Token = u8> {}

pub trait InputBoxableSlice<'src>: InputSlice<'src>
where
    for<'tmp> Self::Slice<'tmp>: BoxableSlice<Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

impl<'src> Input<'src> for &'src str {
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
    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'tmp>>, Self::Error>
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
    type Slice<'tmp> = &'src str;

    fn acquire_slice<'tmp>(&mut self, cursor: Self::Cursor) -> Result<(Self::Slice<'tmp>, bool), Self::Error> {
        todo!()
    }
    fn discard_slice<'tmp>(&mut self, cursor: &mut Self::Cursor, length: usize) -> Self::Slice<'tmp> {
        *self
    }
}

//------------------------------------------------------------------------------

use alloc::string::String;

impl<'src> Input<'src> for String {
    type Token = char;
    type TokenMaybe<'tmp>
        = char
    where
        'src: 'tmp;

    type Error = Infallible;
    type Cursor = usize;

    fn begin(&self) -> Self::Cursor {
        0
    }
    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'tmp>>, Self::Error>
    where
        'src: 'tmp,
    {
        Ok(Some('A'))
    }
    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        todo!()
    }
    fn offset(&self, cursor: Self::Cursor) -> usize {
        todo!()
    }
}

impl<'src> InputSlice<'src> for String {
    type Slice<'tmp> = &'tmp str;

    fn acquire_slice<'tmp>(&mut self, cursor: Self::Cursor) -> Result<(Self::Slice<'tmp>, bool), Self::Error> {
        todo!()
    }
    fn discard_slice<'tmp>(&mut self, cursor: &mut Self::Cursor, length: usize) -> Self::Slice<'tmp> {
        unsafe { core::mem::transmute(self.as_str()) }
    }
}
