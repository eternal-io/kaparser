use core::{
    num::NonZeroUsize,
    ops::{Deref, Range},
};

pub trait Slice {
    type Item: Copy + PartialEq;

    fn len(&self) -> usize;
    fn len_of(item: Self::Item) -> usize;

    fn split_at(&self, mid: usize) -> (&Self, &Self);
    fn subslice(&self, range: Range<usize>) -> &Self;

    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item>;
    fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)>;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    fn first(&self) -> Option<Self::Item> {
        self.iter().next()
    }

    #[inline]
    fn before(&self, off: usize) -> &Self {
        self.split_at(off).0
    }
    #[inline]
    fn after(&self, off: usize) -> &Self {
        self.split_at(off).1
    }

    #[inline]
    fn starts_with(&self, prefix: &Self, ended: bool) -> Result<usize, Result<Option<NonZeroUsize>, usize>> {
        if self.len() < prefix.len() {
            match ended {
                true => Err(Err(self.len())),
                false => Err(Ok(Some((prefix.len() - self.len()).try_into().unwrap()))),
            }
        } else {
            for ((off, item), expected) in self.iter_indices().zip(prefix.iter()) {
                if item != expected {
                    return Err(Err(off));
                }
            }
            Ok(prefix.len())
        }
    }
}

impl Slice for str {
    type Item = char;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(item: Self::Item) -> usize {
        item.len_utf8()
    }

    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline]
    fn subslice(&self, range: Range<usize>) -> &Self {
        &self[range]
    }

    #[inline]
    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item> {
        (*self).chars()
    }
    #[inline]
    fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
        (*self).char_indices()
    }
}

impl<T> Slice for [T]
where
    T: Copy + PartialEq,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(item: Self::Item) -> usize {
        let _ = item;
        1
    }

    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline]
    fn subslice(&self, range: Range<usize>) -> &Self {
        &self[range]
    }

    #[inline]
    fn iter(&self) -> impl DoubleEndedIterator<Item = Self::Item> {
        (*self).iter().copied()
    }
    #[inline]
    fn iter_indices(&self) -> impl DoubleEndedIterator<Item = (usize, Self::Item)> {
        (*self).iter().copied().enumerate()
    }
}

//------------------------------------------------------------------------------

pub trait Stream<'i>: Deref<Target = Self::Slice> {
    type Slice: ?Sized + Slice;

    fn rest(&self) -> &'i Self::Slice;
    fn bump(&mut self, n: usize);
    fn consumed(&self) -> usize;
    fn ended(&self) -> bool;
}

impl<'i, U> Stream<'i> for &'i U
where
    U: ?Sized + Slice,
{
    type Slice = U;

    #[inline]
    fn rest(&self) -> &'i Self::Slice {
        self
    }
    #[inline]
    fn bump(&mut self, n: usize) {
        *self = self.after(n);
    }
    #[inline]
    fn consumed(&self) -> usize {
        0
    }
    #[inline]
    fn ended(&self) -> bool {
        true
    }
}

//------------------------------------------------------------------------------

pub trait DynamicSlice<'i, U>
where
    U: ?Sized + Slice + 'i,
{
    fn bump(&mut self, n: usize) -> &'i U;
    fn rest(&self) -> &'i U;
    fn source(&self) -> &'i U;
    fn consumed(&self) -> usize;
}

impl<'i, U> DynamicSlice<'i, U> for &'i U
where
    U: ?Sized + Slice + 'i,
{
    #[inline]
    fn bump(&mut self, n: usize) -> &'i U {
        // let work = self.rest();
        // *self = self.after(n);
        // work
        todo!()
    }
    #[inline]
    fn rest(&self) -> &'i U {
        self
    }
    #[inline]
    fn source(&self) -> &'i U {
        self
    }
    #[inline]
    fn consumed(&self) -> usize {
        0
    }
}

impl<'i, U> DynamicSlice<'i, U> for StatefulSlice<'i, U>
where
    U: ?Sized + Slice + 'i,
{
    #[inline]
    fn bump(&mut self, n: usize) -> &'i U {
        // let work = self.rest();
        // self.consumed += n;
        // work
        todo!()
    }
    #[inline]
    fn rest(&self) -> &'i U {
        self.source.after(self.consumed)
    }
    #[inline]
    fn source(&self) -> &'i U {
        self.source
    }
    #[inline]
    fn consumed(&self) -> usize {
        self.consumed
    }
}

pub struct StatefulSlice<'i, U>
where
    U: ?Sized + Slice + 'i,
{
    source: &'i U,
    consumed: usize,
}

//------------------------------------------------------------------------------

pub trait ThinSlice: Slice {
    fn as_bytes(&self) -> &[u8];
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool;

    fn memchr1_impl(&self, a: Self::Item) -> Option<(usize, Self::Item)>;
    fn memchr2_impl(&self, a: Self::Item, b: Self::Item) -> Option<(usize, Self::Item)>;
    fn memchr3_impl(&self, a: Self::Item, b: Self::Item, c: Self::Item) -> Option<(usize, Self::Item)>;

    #[inline]
    fn memchr<X: Needlable<Self>>(&self, needle: X) -> Option<(usize, Self::Item)> {
        needle.memchr_invoke(self)
    }

    #[inline]
    fn memmem(&self, needle: &Self) -> Option<usize> {
        memchr::memmem::find(self.as_bytes(), needle.as_bytes())
    }
}

impl ThinSlice for str {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
    #[inline]
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool {
        left.eq_ignore_ascii_case(&right)
    }

    #[inline]
    fn memchr1_impl(&self, a: Self::Item) -> Option<(usize, Self::Item)> {
        let haystack = self.as_bytes();
        let indicator = encode_utf8_first_byte(a);
        let mut offset = 0;
        while let Some(pos) = memchr::memchr(indicator, &haystack[offset..]) {
            offset += pos;
            if next_code_point(&haystack[offset..]) == a as u32 {
                return Some((offset, a));
            }
            offset += 1;
        }
        None
    }
    #[inline]
    fn memchr2_impl(&self, a: Self::Item, b: Self::Item) -> Option<(usize, Self::Item)> {
        let haystack = self.as_bytes();
        let indicator1 = encode_utf8_first_byte(a);
        let indicator2 = encode_utf8_first_byte(b);
        let mut offset = 0;
        while let Some(pos) = memchr::memchr2(indicator1, indicator2, &haystack[offset..]) {
            offset += pos;
            match next_code_point(&haystack[offset..]) {
                needle if needle == a as u32 => return Some((offset, a)),
                needle if needle == b as u32 => return Some((offset, b)),
                _ => offset += 1,
            }
        }
        None
    }
    #[inline]
    fn memchr3_impl(&self, a: Self::Item, b: Self::Item, c: Self::Item) -> Option<(usize, Self::Item)> {
        let haystack = self.as_bytes();
        let indicator1 = encode_utf8_first_byte(a);
        let indicator2 = encode_utf8_first_byte(b);
        let indicator3 = encode_utf8_first_byte(c);
        let mut offset = 0;
        while let Some(pos) = memchr::memchr3(indicator1, indicator2, indicator3, &haystack[offset..]) {
            offset += pos;
            match next_code_point(&haystack[offset..]) {
                needle if needle == a as u32 => return Some((offset, a)),
                needle if needle == b as u32 => return Some((offset, b)),
                needle if needle == c as u32 => return Some((offset, c)),
                _ => offset += 1,
            }
        }
        None
    }
}

impl ThinSlice for [u8] {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }
    #[inline]
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool {
        left.eq_ignore_ascii_case(&right)
    }

    #[inline]
    fn memchr1_impl(&self, a: Self::Item) -> Option<(usize, Self::Item)> {
        Some((memchr::memchr(a, self)?, a))
    }
    #[inline]
    #[allow(unsafe_code)]
    fn memchr2_impl(&self, a: Self::Item, b: Self::Item) -> Option<(usize, Self::Item)> {
        let pos = memchr::memchr2(a, b, self)?;
        Some(match self[pos] {
            needle if needle == a => (pos, a),
            needle if needle == b => (pos, b),
            _ => unsafe { core::hint::unreachable_unchecked() },
        })
    }
    #[inline]
    #[allow(unsafe_code)]
    fn memchr3_impl(&self, a: Self::Item, b: Self::Item, c: Self::Item) -> Option<(usize, Self::Item)> {
        let pos = memchr::memchr3(a, b, c, self)?;
        Some(match self[pos] {
            needle if needle == a => (pos, a),
            needle if needle == b => (pos, b),
            needle if needle == c => (pos, c),
            _ => unsafe { core::hint::unreachable_unchecked() },
        })
    }
}

//------------------------------------------------------------------------------

pub trait Needlable<U: ?Sized + ThinSlice>: Copy {
    fn memchr_invoke(&self, haystack: &U) -> Option<(usize, U::Item)>;
}

impl<U: ?Sized + ThinSlice> Needlable<U> for [U::Item; 1] {
    #[inline]
    fn memchr_invoke(&self, haystack: &U) -> Option<(usize, U::Item)> {
        haystack.memchr1_impl(self[0])
    }
}
impl<U: ?Sized + ThinSlice> Needlable<U> for [U::Item; 2] {
    #[inline]
    fn memchr_invoke(&self, haystack: &U) -> Option<(usize, U::Item)> {
        haystack.memchr2_impl(self[0], self[1])
    }
}
impl<U: ?Sized + ThinSlice> Needlable<U> for [U::Item; 3] {
    #[inline]
    fn memchr_invoke(&self, haystack: &U) -> Option<(usize, U::Item)> {
        haystack.memchr3_impl(self[0], self[1], self[2])
    }
}

//------------------------------------------------------------------------------
// Reference: rust-src (str_internals)

const CONT_MASK: u8 = 0b0011_1111;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;

#[inline]
const fn encode_utf8_first_byte(ch: char) -> u8 {
    let code = ch as u32;
    match ch.len_utf8() {
        1 => code as u8,
        2 => (code >> 6 & 0x1F) as u8 | TAG_TWO_B,
        3 => (code >> 12 & 0x0F) as u8 | TAG_THREE_B,
        4 => (code >> 18 & 0x07) as u8 | TAG_FOUR_B,
        _ => unreachable!(),
    }
}

#[inline]
const fn decode_utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

#[inline]
const fn decode_utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

/// # SAFETY
///
/// The `bytes` must be valid UTF-8.
#[inline]
#[allow(unsafe_code)]
fn next_code_point(bytes: &[u8]) -> u32 {
    let x = unsafe { *bytes.get_unchecked(0) };
    if x < 128 {
        return x as u32;
    }

    // Multibyte case follows
    // Decode from a byte combination out of: [[[x y] z] w]
    let init = decode_utf8_first_byte(x, 2);
    let y = unsafe { *bytes.get_unchecked(1) };
    let mut ch = decode_utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        let z = unsafe { *bytes.get_unchecked(2) };
        let y_z = decode_utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            let w = unsafe { *bytes.get_unchecked(3) };
            ch = (init & 7) << 18 | decode_utf8_acc_cont_byte(y_z, w);
        }
    }

    ch
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VECTOR: &str = "Hello🌍! 测试: 中文「你好」+ 日文「こんにちは」+ 韩文「안녕하세요」 ∑(π²) ≠ √∞ €¥₹$ Āãêī [] ♔♛♝ ♠♥♦♣ ก้้้้้้้้้้้้้้้้้้้ 𐐷 ( ͡° ͜ʖ ͡°) ⟰⏳⌘";

    #[test]
    fn test_utf8_first_byte() {
        let mut buf = [0u8; 4];
        for ch in TEST_VECTOR.chars() {
            ch.encode_utf8(buf.as_mut());
            assert_eq!(encode_utf8_first_byte(ch), buf[0]);
        }
    }

    #[test]
    fn test_next_code_point() {
        for (pos, ch) in TEST_VECTOR.char_indices() {
            assert_eq!(next_code_point(TEST_VECTOR.after(pos).as_ref()), ch as u32);
        }
    }

    #[test]
    fn test_memchr_utf8() {
        for ch in TEST_VECTOR.chars() {
            assert_eq!(TEST_VECTOR.memchr1_impl(ch).unwrap().0, TEST_VECTOR.find(ch).unwrap());
        }
    }

    #[test]
    fn test_memchr() {
        for &byte in TEST_VECTOR.as_bytes() {
            assert_eq!(
                TEST_VECTOR.as_bytes().memchr1_impl(byte).unwrap().0,
                TEST_VECTOR.as_bytes().iter().position(|b| *b == byte).unwrap(),
            )
        }
    }
}
