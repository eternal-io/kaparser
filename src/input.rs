use crate::{common::*, marker, slice::*};
use core::{convert::Infallible, fmt::Debug, ops::Range};

pub trait Input<'src>: 'src {
    type _Marker;

    type Token: 'src;
    type TokenMaybe<'tmp>: RefVal<'tmp, Self::Token>
    where
        'src: 'tmp;

    type Error: Debug;
    type Cursor: Clone;

    fn begin(&self) -> Self::Cursor;

    // DESIGN: Returns `None` if out of index.
    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: Self::Cursor,
    ) -> Result<Option<(Self::Cursor, Self::TokenMaybe<'tmp>)>, Self::Error>
    where
        'src: 'tmp;

    fn span(&self, range: Range<Self::Cursor>) -> Range<usize>;

    fn offset(&self, cursor: Self::Cursor) -> usize;
}

pub trait InputBorrowableToken<'src>: Input<'src> {
    fn get_borrowed(&self, cursor: Self::Cursor) -> Option<&'src Self::Token>;
}

pub trait InputOwnableToken<'src>: Input<'src> {
    fn get_owned(&mut self, cursor: Self::Cursor) -> Option<Self::Token>;
}

impl<'src, I> InputOwnableToken<'src> for I
where
    I: Input<'src>,
    I::Token: Clone,
{
    fn get_owned(&mut self, cursor: Self::Cursor) -> Option<Self::Token> {
        self.next_maybe_ref(cursor)
            .unwrap()
            .map(|(_cursor, item)| item.cloned())
    }
}

pub trait InputSlice<'src>: Input<'src> {
    type Slice: ?Sized + Slice<'src, Item = Self::Token> + 'src;

    fn fetch_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp;

    fn discard_slice<'tmp>(&'tmp mut self, start: Self::Cursor, length: usize) -> (&'tmp Self::Slice, Self::Cursor)
    where
        'src: 'tmp;
}

pub trait InputByteSlice<'src>: InputSlice<'src> {
    fn fetch_byte_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp [u8], bool), Self::Error>
    where
        'src: 'tmp;
}

#[cfg(feature = "alloc")]
pub trait InputBoxableSlice<'src>: InputSlice<'src>
where
    Self::Slice: BoxableSlice<'src, Item = Self::Token>,
{
}

#[cfg(feature = "alloc")]
impl<'src, I> InputBoxableSlice<'src> for I
where
    I: InputSlice<'src>,
    I::Slice: BoxableSlice<'src, Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

impl<'src> Input<'src> for &'src str {
    type _Marker = marker::StaticInput;

    type Token = char;
    type TokenMaybe<'tmp>
        = char
    where
        'src: 'tmp;

    type Error = Infallible;
    type Cursor = usize;

    #[inline]
    fn begin(&self) -> Self::Cursor {
        0
    }

    #[inline]
    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: Self::Cursor,
    ) -> Result<Option<(Self::Cursor, Self::TokenMaybe<'tmp>)>, Self::Error>
    where
        'src: 'tmp,
    {
        Ok(self.after(cursor).iter_indices().next())
    }

    #[inline]
    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        assert!(self.is_item_boundary(range.start));
        assert!(self.is_item_boundary(range.end));
        range
    }

    #[inline]
    fn offset(&self, cursor: Self::Cursor) -> usize {
        assert!(self.is_item_boundary(cursor));
        cursor
    }
}

impl<'src> InputSlice<'src> for &'src str {
    type Slice = str;

    #[inline]
    fn fetch_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp,
    {
        Ok((self.after(start), true))
    }

    #[inline]
    fn discard_slice<'tmp>(&'tmp mut self, start: Self::Cursor, length: usize) -> (&'tmp Self::Slice, Self::Cursor)
    where
        'src: 'tmp,
    {
        let end = start + length;
        (self.subslice(start..end), end)
    }
}

impl<'src> InputByteSlice<'src> for &'src str {
    fn fetch_byte_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp [u8], bool), Self::Error>
    where
        'src: 'tmp,
    {
        Ok((self.after(start).as_bytes(), true))
    }
}

//------------------------------------------------------------------------------

impl<'src, T> Input<'src> for &'src [T] {
    type _Marker = marker::StaticInput;

    type Token = T;
    type TokenMaybe<'tmp>
        = &'tmp T
    where
        'src: 'tmp;

    type Error = Infallible;
    type Cursor = usize;

    #[inline]
    fn begin(&self) -> Self::Cursor {
        0
    }

    #[inline]
    fn next_maybe_ref<'tmp>(
        &'tmp mut self,
        cursor: Self::Cursor,
    ) -> Result<Option<(Self::Cursor, Self::TokenMaybe<'tmp>)>, Self::Error>
    where
        'src: 'tmp,
    {
        Ok(self.after(cursor).iter_indices().next())
    }

    #[inline]
    fn span(&self, range: Range<Self::Cursor>) -> Range<usize> {
        assert!(self.is_item_boundary(range.start));
        assert!(self.is_item_boundary(range.end));
        range
    }

    #[inline]
    fn offset(&self, cursor: Self::Cursor) -> usize {
        assert!(self.is_item_boundary(cursor));
        cursor
    }
}

impl<'src, T> InputBorrowableToken<'src> for &'src [T] {
    fn get_borrowed(&self, cursor: Self::Cursor) -> Option<&'src Self::Token> {
        self.get(cursor)
    }
}

impl<'src, T: 'src> InputSlice<'src> for &'src [T] {
    type Slice = [T];

    #[inline]
    fn fetch_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), Self::Error>
    where
        'src: 'tmp,
    {
        Ok((self.after(start), true))
    }

    #[inline]
    fn discard_slice<'tmp>(&'tmp mut self, start: Self::Cursor, length: usize) -> (&'tmp Self::Slice, Self::Cursor)
    where
        'src: 'tmp,
    {
        let end = start + length;
        (self.subslice(start..end), end)
    }
}

impl<'src> InputByteSlice<'src> for &'src [u8] {
    fn fetch_byte_slice<'tmp>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp [u8], bool), Self::Error>
    where
        'src: 'tmp,
    {
        self.fetch_slice(start)
    }
}
