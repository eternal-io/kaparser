use crate::common::*;
use core::ops::Range;

pub trait Slice<'src>: 'src {
    type Item: 'src;
    type ItemMaybe<'tmp>: RefVal<'tmp, Self::Item>
    where
        'src: 'tmp;

    fn len(&self) -> usize;
    fn len_of(&self, item: &Self::Item) -> usize;

    fn subslice(&self, range: Range<usize>) -> &Self;
    fn split_at(&self, mid: usize) -> (&Self, &Self);

    fn iter<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = Self::ItemMaybe<'tmp>>
    where
        'src: 'tmp;
    fn iter_indices<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = (usize, Self::ItemMaybe<'tmp>)>
    where
        'src: 'tmp;

    fn is_item_boundary(&self, idx: usize) -> bool;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn first<'tmp>(&'tmp self) -> Option<Self::ItemMaybe<'tmp>>
    where
        'src: 'tmp,
    {
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
    type ItemMaybe<'tmp>
        = char
    where
        'src: 'tmp;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(&self, item: &Self::Item) -> usize {
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
    fn iter<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = Self::ItemMaybe<'tmp>>
    where
        'src: 'tmp,
    {
        (*self).chars()
    }
    #[inline]
    fn iter_indices<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = (usize, Self::ItemMaybe<'tmp>)>
    where
        'src: 'tmp,
    {
        (*self).char_indices()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        (*self).is_char_boundary(idx)
    }
}

impl<'src, T: 'src> Slice<'src> for [T] {
    type Item = T;
    type ItemMaybe<'tmp>
        = &'tmp T
    where
        'src: 'tmp;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(&self, item: &Self::Item) -> usize {
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
    fn iter<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = Self::ItemMaybe<'tmp>>
    where
        'src: 'tmp,
    {
        (*self).iter()
    }
    #[inline]
    fn iter_indices<'tmp>(&'tmp self) -> impl DoubleEndedIterator<Item = (usize, Self::ItemMaybe<'tmp>)>
    where
        'src: 'tmp,
    {
        (*self).iter().enumerate()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        idx <= self.len()
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
