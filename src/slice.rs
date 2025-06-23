use core::ops::Range;

pub trait Slice<'src> {
    type Item;

    fn len(&self) -> usize;
    fn len_of(&self, item: Self::Item) -> usize;

    fn subslice(&self, range: Range<usize>) -> &Self;
    fn split_at(&self, mid: usize) -> (&Self, &Self);

    fn iter(&'src self) -> impl DoubleEndedIterator<Item = Self::Item>;
    fn iter_indices(&'src self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)>;

    fn is_item_boundary(&self, idx: usize) -> bool;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn first(&'src self) -> Option<Self::Item> {
        self.iter().next()
    }

    #[inline]
    fn after(&self, off: usize) -> &Self {
        self.split_at(off).1
    }
    #[inline]
    fn before(&self, off: usize) -> &Self {
        self.split_at(off).0
    }
}

impl<'src> Slice<'src> for str {
    type Item = char;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(&self, item: Self::Item) -> usize {
        item.len_utf8()
    }

    #[inline]
    fn subslice(&self, range: Range<usize>) -> &Self {
        &self[range]
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }

    #[inline]
    fn iter(&'src self) -> impl DoubleEndedIterator<Item = Self::Item> {
        (*self).chars()
    }
    #[inline]
    fn iter_indices(&'src self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
        (*self).char_indices()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        (*self).is_char_boundary(idx)
    }
}

impl<'src, T: 'src> Slice<'src> for [T] {
    type Item = &'src T;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(&self, item: Self::Item) -> usize {
        #![allow(unused_variables)]
        1
    }

    #[inline]
    fn subslice(&self, range: Range<usize>) -> &Self {
        &self[range]
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }

    #[inline]
    fn iter(&'src self) -> impl DoubleEndedIterator<Item = Self::Item> {
        (*self).iter()
    }
    #[inline]
    fn iter_indices(&'src self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
        (*self).iter().enumerate()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        #![allow(unused_variables)]
        true
    }
}

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
pub trait BoxableSlice<'src>: Slice<'src> {
    fn to_boxed(&self) -> Box<Self>;
}

#[cfg(feature = "alloc")]
impl<'src> BoxableSlice<'src> for str {
    fn to_boxed(&self) -> Box<Self> {
        Box::from(self)
    }
}

#[cfg(feature = "alloc")]
impl<'src, T: 'src + Clone> BoxableSlice<'src> for [T] {
    fn to_boxed(&self) -> Box<Self> {
        Box::from(self)
    }
}
