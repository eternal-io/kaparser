use core::{
    iter::{Copied, Enumerate},
    num::NonZeroUsize,
    ops::Range,
    slice::Iter,
    str::{CharIndices, Chars},
};

pub trait Slice {
    type Item: Copy + PartialEq;

    type Iter<'i>: DoubleEndedIterator<Item = Self::Item>
    where
        Self: 'i;

    type IterIndices<'i>: DoubleEndedIterator<Item = (usize, Self::Item)>
    where
        Self: 'i;

    fn len(&self) -> usize;
    fn len_of(item: Self::Item) -> usize;

    fn subslice(&self, range: Range<usize>) -> &Self;
    fn split_at(&self, mid: usize) -> (&Self, &Self);

    fn iter(&self) -> Self::Iter<'_>;
    fn iter_indices(&self) -> Self::IterIndices<'_>;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    fn first(&self) -> Option<Self::Item> {
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

    type Iter<'i>
        = Chars<'i>
    where
        Self: 'i;

    type IterIndices<'i>
        = CharIndices<'i>
    where
        Self: 'i;

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn len_of(item: Self::Item) -> usize {
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
    fn iter(&self) -> Self::Iter<'_> {
        (*self).chars()
    }
    #[inline]
    fn iter_indices(&self) -> Self::IterIndices<'_> {
        (*self).char_indices()
    }
}

impl<T> Slice for [T]
where
    T: Copy + PartialEq,
{
    type Item = T;

    type Iter<'i>
        = Copied<Iter<'i, T>>
    where
        Self: 'i;

    type IterIndices<'i>
        = Enumerate<Copied<Iter<'i, T>>>
    where
        Self: 'i;

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
    fn subslice(&self, range: Range<usize>) -> &Self {
        &self[range]
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter().copied()
    }
    #[inline]
    fn iter_indices(&self) -> Self::IterIndices<'_> {
        (*self).iter().copied().enumerate()
    }
}

//------------------------------------------------------------------------------

pub trait Stream<'i, U>
where
    U: ?Sized + Slice + 'i,
{
    fn rest(&self) -> &'i U;
    fn bump(&mut self, n: usize) -> &'i U;
    fn consumed(&self) -> usize;
    fn ended(&self) -> bool;

    /*
        Delegation is needed since deref cannot meet the requirements.
        It is also desirable to implement these methods in two traits,
        as this allows for chaining of methods on slices.
    */

    #[inline]
    fn len(&self) -> usize {
        self.rest().len()
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.rest().is_empty()
    }

    #[inline]
    fn first(&self) -> Option<U::Item> {
        self.rest().iter().next()
    }
    #[inline]
    fn iter(&self) -> U::Iter<'i> {
        self.rest().iter()
    }
    #[inline]
    fn iter_indices(&self) -> U::IterIndices<'i> {
        self.rest().iter_indices()
    }

    #[inline]
    fn after(&self, off: usize) -> &'i U {
        self.rest().after(off)
    }
    #[inline]
    fn before(&self, off: usize) -> &'i U {
        self.rest().before(off)
    }
    #[inline]
    fn subslice(&self, range: Range<usize>) -> &'i U {
        self.rest().subslice(range)
    }
    #[inline]
    fn split_at(&self, mid: usize) -> (&'i U, &'i U) {
        self.rest().split_at(mid)
    }

    #[inline]
    fn starts_with(&self, prefix: &U, ended: bool) -> Result<usize, Result<Option<NonZeroUsize>, usize>> {
        self.rest().starts_with(prefix, ended)
    }
}

impl<'i, U> Stream<'i, U> for &'i U
where
    U: ?Sized + Slice,
{
    #[inline]
    fn rest(&self) -> &'i U {
        self
    }
    #[inline]
    fn bump(&mut self, n: usize) -> &'i U {
        let past = *self;
        *self = self.after(n);
        past
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
