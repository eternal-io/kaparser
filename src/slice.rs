#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ops::Range;

/*
pub trait Slice: Sized {
    type Item<'a>;
    type Slice: ?Sized + 'static;

    fn len(&self) -> usize;
    fn len_of<'a>(&self, item: &Self::Item<'a>) -> usize;

    fn subslice(&self, range: Range<usize>) -> Self;
    fn split_at(&self, mid: usize) -> (Self, Self);

    fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = Self::Item<'a>>;
    fn iter_indices<'a>(&'a self) -> impl DoubleEndedIterator<Item = (usize, Self::Item<'a>)>;

    fn is_item_boundary(&self, idx: usize) -> bool;

    #[cfg(feature = "alloc")]
    fn to_owned(&self) -> Box<Self::Slice>
    where
        for<'a> Self::Item<'a>: Clone;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn first<'a>(&'a self) -> Option<Self::Item<'a>> {
        self.iter().next()
    }

    #[inline]
    fn after(&self, off: usize) -> Self {
        self.split_at(off).1
    }
    #[inline]
    fn before(&self, off: usize) -> Self {
        self.split_at(off).0
    }
}

impl Slice for &str {
    type Item<'a> = char;
    type Slice = str;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of<'a>(&self, item: &Self::Item<'a>) -> usize {
        item.len_utf8()
    }

    #[inline]
    fn subslice(&self, range: Range<usize>) -> Self {
        &self[range]
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (Self, Self) {
        (*self).split_at(mid)
    }

    #[inline]
    fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = Self::Item<'a>> {
        (*self).chars()
    }
    #[inline]
    fn iter_indices<'a>(&'a self) -> impl DoubleEndedIterator<Item = (usize, Self::Item<'a>)> {
        (*self).char_indices()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        (*self).is_char_boundary(idx)
    }

    #[cfg(feature = "alloc")]
    fn to_owned(&self) -> Box<Self::Slice>
    where
        for<'a> Self::Item<'a>: Clone,
    {
        Box::from(*self)
    }
}
*/

pub trait Slice: Sized {
    type Item;
    type Slice: ?Sized;

    fn len(&self) -> usize;
    fn len_of(&self, item: Self::Item) -> usize;

    fn subslice(&self, range: Range<usize>) -> Self;
    fn split_at(&self, mid: usize) -> (Self, Self);

    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item>;
    fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)>;

    fn is_item_boundary(&self, idx: usize) -> bool;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn first(&self) -> Option<Self::Item> {
        self.iter().next()
    }

    #[inline]
    fn after(&self, off: usize) -> Self {
        self.split_at(off).1
    }
    #[inline]
    fn before(&self, off: usize) -> Self {
        self.split_at(off).0
    }
}

impl Slice for &str {
    type Item = char;
    type Slice = str;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(&self, item: Self::Item) -> usize {
        item.len_utf8()
    }

    #[inline]
    fn subslice(&self, range: Range<usize>) -> Self {
        &self[range]
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (Self, Self) {
        (*self).split_at(mid)
    }

    #[inline]
    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item> {
        (*self).chars()
    }
    #[inline]
    fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
        (*self).char_indices()
    }

    #[inline]
    fn is_item_boundary(&self, idx: usize) -> bool {
        (*self).is_char_boundary(idx)
    }
}

// impl<'src, T> Slice for &'src [T] {
//     type Item = &'src T;
//     type Slice = [T];

//     #[inline]
//     fn len(&self) -> usize {
//         (*self).len()
//     }
//     #[inline]
//     fn len_of(&self, item: Self::Item) -> usize {
//         #![allow(unused_variables)]
//         1
//     }

//     #[inline]
//     fn subslice(&self, range: Range<usize>) -> Self {
//         &self[range]
//     }
//     #[inline]
//     fn split_at(&self, mid: usize) -> (Self, Self) {
//         (*self).split_at(mid)
//     }

//     #[inline]
//     fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item> {
//         (*self).iter()
//     }
//     #[inline]
//     fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
//         (*self).iter().enumerate()
//     }

//     #[inline]
//     fn is_item_boundary(&self, idx: usize) -> bool {
//         #![allow(unused_variables)]
//         true
//     }
// }

#[cfg(feature = "alloc")]
pub trait BoxableSlice: Slice {
    fn to_boxed(&self) -> Box<Self>;
}

#[cfg(feature = "alloc")]
impl BoxableSlice for &str {
    fn to_boxed(&self) -> Box<Self> {
        Box::from(*self)
    }
}

// #[cfg(feature = "alloc")]
// impl<T: Clone> BoxableSlice for &[T] {
//     fn to_boxed(&self) -> Box<Self> {
//         Box::from(*self)
//     }
// }
