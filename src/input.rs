use crate::{common::*, error::*, slice::*};
use core::ops::Range;

// TODO: In other words, they are inputs that do not need to be mutated when getting a slice or item.
pub unsafe trait StaticInput {}

pub trait Input<'src>: 'src {
    type Token: 'src;

    type TokenMaybe<'tmp>: RefVal<'tmp, Self::Token>
    where
        'src: 'tmp;

    type Cursor: Clone;

    fn begin(&self) -> Self::Cursor;

    fn next_maybe_ref<'tmp, E: Error>(
        &'tmp mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'tmp>>, E>
    where
        'src: 'tmp;

    fn has_reached_end(&mut self, cursor: Self::Cursor) -> bool;

    fn shall_reached_end<E: Error>(&mut self, cursor: Self::Cursor) -> Option<E> {
        (!self.has_reached_end(cursor.clone())).then(|| E::new(Self::offset_span(cursor), ErrorKind::ExpectedEnd))
    }

    fn span(range: Range<Self::Cursor>) -> Range<usize>;

    fn offset(cursor: Self::Cursor) -> usize;

    fn offset_span(cursor: Self::Cursor) -> Range<usize> {
        let o = Self::offset(cursor);
        o..o
    }
}

pub trait InputOwnableToken<'src>: Input<'src> {
    fn get_owned(&self, cursor: Self::Cursor) -> Option<Self::Token>;

    fn iter_owned(&self, range: Range<Self::Cursor>) -> impl Iterator<Item = Self::Token>;
}

pub trait InputBorrowableToken<'src>: Input<'src> {
    fn get_borrowed(&self, cursor: Self::Cursor) -> Option<&'src Self::Token>;

    fn iter_borrowed(&self, range: Range<Self::Cursor>) -> impl Iterator<Item = &'src Self::Token>;
}

pub trait InputSlice<'src>: Input<'src> {
    type Slice: ?Sized + Slice<'src, Item = Self::Token>;

    fn get_slice<'tmp>(&'tmp self, range: Range<Self::Cursor>) -> Option<&'tmp Self::Slice>
    where
        'src: 'tmp;

    fn fetch_slice<'tmp, E: Error>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), E>
    where
        'src: 'tmp;

    fn release_slice<'tmp>(&'tmp mut self, range: Range<Self::Cursor>) -> &'tmp Self::Slice
    where
        'src: 'tmp;

    fn bump_cursor(cursor: Self::Cursor, length: usize) -> Self::Cursor;
}

pub trait InputByteSlice<'src>: InputSlice<'src> {
    fn fetch_byte_slice<'tmp, E: Error>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp [u8], bool), E>
    where
        'src: 'tmp;
}

#[cfg(feature = "alloc")]
pub trait InputBoxableSlice<'src>: InputSlice<'src>
where
    Self::Slice: BoxableSlice<'src, Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

impl<'src, I> InputByteSlice<'src> for I
where
    I: InputSlice<'src>,
    I::Slice: AsRef<[u8]>,
{
    fn fetch_byte_slice<'tmp, E: Error>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp [u8], bool), E>
    where
        'src: 'tmp,
    {
        self.fetch_slice(start).map(|(slice, eof)| (slice.as_ref(), eof))
    }
}

#[cfg(feature = "alloc")]
impl<'src, I> InputBoxableSlice<'src> for I
where
    I: InputSlice<'src>,
    I::Slice: BoxableSlice<'src, Item = Self::Token>,
{
}

//------------------------------------------------------------------------------

unsafe impl<'src, S> StaticInput for &'src S where S: ?Sized + Slice<'src> {}

impl<'src, S> Input<'src> for &'src S
where
    S: ?Sized + Slice<'src>,
{
    type Token = S::Item;

    type TokenMaybe<'tmp>
        = S::ItemMaybe<'tmp>
    where
        'src: 'tmp;

    type Cursor = usize;

    #[inline]
    fn begin(&self) -> Self::Cursor {
        0
    }

    #[inline]
    fn next_maybe_ref<'tmp, E: Error>(
        &'tmp mut self,
        cursor: &mut Self::Cursor,
    ) -> Result<Option<Self::TokenMaybe<'tmp>>, E>
    where
        'src: 'tmp,
    {
        Ok(self
            .after(*cursor)
            .first()
            .inspect(|item| *cursor += S::len_of(item.as_ref())))
    }

    #[inline]
    fn has_reached_end(&mut self, cursor: Self::Cursor) -> bool {
        debug_assert!(self.is_item_boundary(cursor));
        cursor == self.len()
    }

    #[inline]
    fn span(range: Range<Self::Cursor>) -> Range<usize> {
        range
    }

    #[inline]
    fn offset(cursor: Self::Cursor) -> usize {
        cursor
    }
}

impl<'src, S> InputSlice<'src> for &'src S
where
    S: ?Sized + Slice<'src>,
{
    type Slice = S;

    #[inline]
    fn get_slice<'tmp>(&'tmp self, range: Range<Self::Cursor>) -> Option<&'tmp Self::Slice>
    where
        'src: 'tmp,
    {
        (self.is_item_boundary(range.start) && self.is_item_boundary(range.end)).then(|| self.subslice(range))
    }

    #[inline]
    fn fetch_slice<'tmp, E: Error>(&'tmp mut self, start: Self::Cursor) -> Result<(&'tmp Self::Slice, bool), E>
    where
        'src: 'tmp,
    {
        Ok((self.after(start), true))
    }

    #[inline]
    fn release_slice<'tmp>(&'tmp mut self, range: Range<Self::Cursor>) -> &'tmp Self::Slice
    where
        'src: 'tmp,
    {
        debug_assert!(self.is_item_boundary(range.start));
        debug_assert!(self.is_item_boundary(range.end));

        self.subslice(range)
    }

    #[inline]
    fn bump_cursor(cursor: Self::Cursor, length: usize) -> Self::Cursor {
        cursor + length
    }
}

impl<'src, S> InputOwnableToken<'src> for &'src S
where
    S: ?Sized + Slice<'src>,
    S::Item: Clone,
{
    fn get_owned(&self, cursor: Self::Cursor) -> Option<Self::Token> {
        self.after(cursor).first().map(|item| item.cloned())
    }

    fn iter_owned(&self, range: Range<Self::Cursor>) -> impl Iterator<Item = Self::Token> {
        self.subslice(range).iter().map(|item| item.cloned())
    }
}

impl<'src, T> InputBorrowableToken<'src> for &'src [T] {
    fn get_borrowed(&self, cursor: Self::Cursor) -> Option<&'src Self::Token> {
        self.get(cursor)
    }

    fn iter_borrowed(&self, range: Range<Self::Cursor>) -> impl Iterator<Item = &'src Self::Token> {
        self.subslice(range).iter()
    }
}
