use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

#[doc(hidden)]
pub use paste::paste;

#[inline(always)]
pub(crate) const fn likely(cond: bool) -> bool {
    if !cond {
        cold_path();
    }
    cond
}
#[inline(always)]
pub(crate) const fn unlikely(cond: bool) -> bool {
    if cond {
        cold_path();
    }
    cond
}
#[cold]
#[inline(always)]
pub(crate) const fn cold_path() {}

//------------------------------------------------------------------------------

/// You can abbreviate `n..=n` to `n`.
pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn unfulfilled(&self, times: usize) -> bool;
}

#[rustfmt::skip]
mod urange_bounds {
    use super::*;

    impl URangeBounds for usize {
        fn contains(&self, times: usize) -> bool { times == *self }
        fn unfulfilled(&self, times: usize) -> bool { times < *self }
    }
    impl URangeBounds for RangeFull {
        fn contains(&self, _t: usize) -> bool { true }
        fn unfulfilled(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for RangeFrom<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for Range<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times < *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times < self.end }
    }
}

//------------------------------------------------------------------------------

pub trait Slice {
    type Item: Copy + PartialEq;

    fn len(&self) -> usize;
    fn len_of(item: Self::Item) -> usize;
    fn starts_with(&self, prefix: &Self) -> bool;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn split_at(&self, mid: usize) -> (&Self, &Self);
    fn iter(&self) -> impl Iterator<Item = Self::Item> + DoubleEndedIterator;
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)> + DoubleEndedIterator;
    fn first(&self) -> Option<Self::Item> {
        self.iter().next()
    }
}

impl Slice for str {
    type Item = char;

    #[inline(always)]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline(always)]
    fn len_of(item: Self::Item) -> usize {
        item.len_utf8()
    }
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        (*self).starts_with(prefix)
    }

    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = Self::Item> + DoubleEndedIterator {
        self.chars()
    }
    #[inline(always)]
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)> + DoubleEndedIterator {
        self.char_indices()
    }
}

impl<T> Slice for [T]
where
    T: Copy + PartialEq,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline(always)]
    fn len_of(_tem: Self::Item) -> usize {
        1
    }
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        (*self).starts_with(prefix)
    }

    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = Self::Item> + DoubleEndedIterator {
        (*self).iter().copied()
    }
    #[inline(always)]
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)> + DoubleEndedIterator {
        (*self).iter().copied().enumerate()
    }
}

//------------------------------------------------------------------------------

pub trait ThinSlice: Slice {
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool;

    fn memchr(&self, needle: Self::Item) -> Option<usize>;
    fn memchr2(&self, needle1: Self::Item, needle2: Self::Item) -> Option<(usize, Self::Item)>;
    fn memchr3(&self, needle1: Self::Item, needle2: Self::Item, needle3: Self::Item) -> Option<(usize, Self::Item)>;

    fn memmem(&self, slice: &Self) -> Option<usize>;
}

impl ThinSlice for str {
    #[inline(always)]
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool {
        left.eq_ignore_ascii_case(&right)
    }

    #[inline(always)]
    fn memchr(&self, needle: Self::Item) -> Option<usize> {
        memchr_utf8(needle, self)
    }
    #[inline(always)]
    fn memchr2(&self, needle1: Self::Item, needle2: Self::Item) -> Option<(usize, Self::Item)> {
        memchr2_utf8(needle1, needle2, self)
    }
    #[inline(always)]
    fn memchr3(&self, needle1: Self::Item, needle2: Self::Item, needle3: Self::Item) -> Option<(usize, Self::Item)> {
        memchr3_utf8(needle1, needle2, needle3, self)
    }

    #[inline(always)]
    fn memmem(&self, needle: &Self) -> Option<usize> {
        memchr::memmem::find(self.as_bytes(), needle.as_bytes())
    }
}

impl ThinSlice for [u8] {
    #[inline(always)]
    fn eq_ignore_ascii_case(left: Self::Item, right: Self::Item) -> bool {
        left.eq_ignore_ascii_case(&right)
    }

    #[inline(always)]
    fn memchr(&self, needle: Self::Item) -> Option<usize> {
        memchr(needle, self)
    }
    #[inline(always)]
    fn memchr2(&self, needle1: Self::Item, needle2: Self::Item) -> Option<(usize, Self::Item)> {
        memchr2(needle1, needle2, self)
    }
    #[inline(always)]
    fn memchr3(&self, needle1: Self::Item, needle2: Self::Item, needle3: Self::Item) -> Option<(usize, Self::Item)> {
        memchr3(needle1, needle2, needle3, self)
    }

    #[inline(always)]
    fn memmem(&self, needle: &Self) -> Option<usize> {
        memchr::memmem::find(self, needle)
    }
}

//------------------------------------------------------------------------------
// Reference: rust-src (str_internals)

const CONT_MASK: u8 = 0b0011_1111;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;

#[inline(always)]
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

#[inline(always)]
const fn decode_utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

#[inline(always)]
const fn decode_utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

/// # SAFETY
///
/// The `bytes` must be valid UTF-8.
#[inline(always)]
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

#[inline(always)]
fn memchr_utf8(needle: char, haystack: &str) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let indicator = encode_utf8_first_byte(needle);
    let mut offset = 0;
    while let Some(pos) = memchr::memchr(indicator, &haystack[offset..]) {
        offset += pos;
        if next_code_point(&haystack[offset..]) == needle as u32 {
            return Some(offset);
        }
        offset += 1;
    }
    None
}
#[inline(always)]
fn memchr2_utf8(needle1: char, needle2: char, haystack: &str) -> Option<(usize, char)> {
    let haystack = haystack.as_bytes();
    let indicator1 = encode_utf8_first_byte(needle1);
    let indicator2 = encode_utf8_first_byte(needle2);
    let mut offset = 0;
    while let Some(pos) = memchr::memchr2(indicator1, indicator2, &haystack[offset..]) {
        offset += pos;
        match next_code_point(&haystack[offset..]) {
            needle if needle == needle1 as u32 => return Some((offset, needle1)),
            needle if needle == needle2 as u32 => return Some((offset, needle2)),
            _ => offset += 1,
        }
    }
    None
}
#[inline(always)]
fn memchr3_utf8(needle1: char, needle2: char, needle3: char, haystack: &str) -> Option<(usize, char)> {
    let haystack = haystack.as_bytes();
    let indicator1 = encode_utf8_first_byte(needle1);
    let indicator2 = encode_utf8_first_byte(needle2);
    let indicator3 = encode_utf8_first_byte(needle3);
    let mut offset = 0;
    while let Some(pos) = memchr::memchr3(indicator1, indicator2, indicator3, &haystack[offset..]) {
        offset += pos;
        match next_code_point(&haystack[offset..]) {
            needle if needle == needle1 as u32 => return Some((offset, needle1)),
            needle if needle == needle2 as u32 => return Some((offset, needle2)),
            needle if needle == needle3 as u32 => return Some((offset, needle3)),
            _ => offset += 1,
        }
    }
    None
}

#[inline(always)]
fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    memchr::memchr(needle, haystack)
}
#[inline(always)]
#[allow(unsafe_code)]
fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<(usize, u8)> {
    let pos = memchr::memchr2(needle1, needle2, haystack)?;
    Some(match haystack[pos] {
        needle if needle == needle1 => (pos, needle1),
        needle if needle == needle2 => (pos, needle2),
        _ => unsafe { core::hint::unreachable_unchecked() },
    })
}
#[inline(always)]
#[allow(unsafe_code)]
fn memchr3(needle1: u8, needle2: u8, needle3: u8, haystack: &[u8]) -> Option<(usize, u8)> {
    let pos = memchr::memchr3(needle1, needle2, needle3, haystack)?;
    Some(match haystack[pos] {
        needle if needle == needle1 => (pos, needle1),
        needle if needle == needle2 => (pos, needle2),
        needle if needle == needle3 => (pos, needle3),
        _ => unsafe { core::hint::unreachable_unchecked() },
    })
}

//------------------------------------------------------------------------------

/// `Lens1X` means `LenX - 1`. Always `N < K < M`.
/// `Gen` means "Generic". "Con" means "Converge".
macro_rules! __generate_codes {
    ( $callback:ident $(($($custom:ident) ~ +))? ) => { paste::paste! {
        __generate_codes! {
          @ $callback ;
            0  ~ 1  $(~ ($([< $custom 1  >]) ~ +))? ~ A ~ A ~ 0
            1  ~ 2  $(~ ($([< $custom 2  >]) ~ +))? ~ B ~ A ~ 1
            2  ~ 3  $(~ ($([< $custom 3  >]) ~ +))? ~ C ~ A ~ 2
            3  ~ 4  $(~ ($([< $custom 4  >]) ~ +))? ~ D ~ A ~ 3
            4  ~ 5  $(~ ($([< $custom 5  >]) ~ +))? ~ E ~ A ~ 4
            5  ~ 6  $(~ ($([< $custom 6  >]) ~ +))? ~ F ~ A ~ 5
            6  ~ 7  $(~ ($([< $custom 7  >]) ~ +))? ~ G ~ A ~ 6
            7  ~ 8  $(~ ($([< $custom 8  >]) ~ +))? ~ H ~ A ~ 7
            8  ~ 9  $(~ ($([< $custom 9  >]) ~ +))? ~ I ~ A ~ 8
            9  ~ 10 $(~ ($([< $custom 10 >]) ~ +))? ~ J ~ A ~ 9
            10 ~ 11 $(~ ($([< $custom 11 >]) ~ +))? ~ K ~ A ~ 10
            11 ~ 12 $(~ ($([< $custom 12 >]) ~ +))? ~ L ~ A ~ 11
            12 ~ 13 $(~ ($([< $custom 13 >]) ~ +))? ~ M ~ A ~ 12
            13 ~ 14 $(~ ($([< $custom 14 >]) ~ +))? ~ N ~ A ~ 13
            14 ~ 15 $(~ ($([< $custom 15 >]) ~ +))? ~ O ~ A ~ 14
            15 ~ 16 $(~ ($([< $custom 16 >]) ~ +))? ~ P ~ A ~ 15
            16 ~ 17 $(~ ($([< $custom 17 >]) ~ +))? ~ Q ~ A ~ 16
            17 ~ 18 $(~ ($([< $custom 18 >]) ~ +))? ~ R ~ A ~ 17
            18 ~ 19 $(~ ($([< $custom 19 >]) ~ +))? ~ S ~ A ~ 18
            19 ~ 20 $(~ ($([< $custom 20 >]) ~ +))? ~ T ~ A ~ 19
            20 ~ 21 $(~ ($([< $custom 21 >]) ~ +))? ~ U ~ A ~ 20
            21 ~ 22 $(~ ($([< $custom 22 >]) ~ +))? ~ V ~ A ~ 21
            22 ~ 23 $(~ ($([< $custom 23 >]) ~ +))? ~ W ~ A ~ 22
        }
    } };

    ( @ $callback:ident ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        __generate_codes! {
          @ $callback ;
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        $callback!( $Lens1K, $($OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+ );
        __generate_codes! {
          @ $callback ;
          $($Lens1N ~ $OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
    ) => {};
}

//------------------------------------------------------------------------------

pub(crate) use alts::*;
pub(crate) use checkpoints::*;

pub trait Convergable<A> {
    fn converge(self) -> A;
}

pub mod alts {
    use super::*;

    macro_rules! gen_alternative {
        ( $Len:literal, $($OrdN:literal ~ ($VarN:ident) ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ) => { paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
            pub enum [<Alt $Len>]<$($GenN),+> { $(
            #[doc = "Variant " $OrdN " of " $Len "."]
                $VarN($GenN),
            )+ }

            impl<A> Convergable<A> for [<Alt $Len>]<$($ConN),+> {
                #[inline(always)]
                fn converge(self) -> A {
                    match self { $(
                        Self::$VarN(v) => v,
                    )+ }
                }
            }
        } }
    }

    __generate_codes! { gen_alternative ( Var ) }
}

#[doc(hidden)]
pub mod checkpoints {
    macro_rules! gen_checkpoint {
        ( $Len:literal, $($OrdN:literal ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ) => { paste::paste! {
            #[doc(hidden)]
            #[derive(Clone)]
            pub enum [<Check $Len>] { $(
                [<Point $OrdN>],
            )+ }
        } }
    }

    __generate_codes! { gen_checkpoint }
}

//------------------------------------------------------------------------------

macro_rules! __resume_advance {
    (      $Ent:expr ;
        $( $CaseN:pat => $TurnN:block $ProcN:block )+
    ) => {
        __resume_advance! {
            @LABELING $Ent ;
            'p1  'p2  'p3  'p4  'p5  'p6  'p7  'p8
            'p9  'p10 'p11 'p12 'p13 'p14 'p15 'p16
            'p17 'p18 'p19 'p20 'p21 'p22 'p23 'p24 ;
            $( $CaseN => $TurnN $ProcN )+ ;
        }
    };

    ( @LABELING $Ent:expr ;
           $LabK:lifetime
        $( $LabM:lifetime )* ;
           $CaseK:pat => $TurnK:block $ProcK:block
        $( $CaseM:pat => $TurnM:block $ProcM:block )* ;
        $( $LabN:lifetime:
           $CaseN:pat => $TurnN:block $ProcN:block )*
    ) => {
        __resume_advance! {
            @LABELING $Ent ;
            $( $LabM )* ;
            $( $CaseM => $TurnM $ProcM )* ;
               $LabK:
               $CaseK => $TurnK $ProcK // cases then appear in reverse order.
            $( $LabN:
               $CaseN => $TurnN $ProcN )*
        }
    };

    ( @LABELING $Ent:expr ;
        /* not enough labels */ ;
        $CaseX:pat => $( $tt:tt )*
    ) => {
        ::core::compile_error!("too many cases, only 24 at most")
    };

    ( @LABELING $Ent:expr ;
        $( $LabN:lifetime )* ;
        /* no more unlabeled cases */ ;
        $( $tt:tt )*
    ) => {
        __resume_advance! { @ENTERING $Ent ; ; $( $tt )* }
    };

    ( @ENTERING $Ent:expr ;
        $( $LabN:lifetime: $CaseN:pat => $TurnN:block $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $TurnK:block $ProcK:block
        $( $LabM:lifetime: $CaseM:pat => $TurnM:block $ProcM:block )+
    ) => {
        $LabK: {
            __resume_advance! {
                @ENTERING $Ent ;
                   $LabK: $CaseK => $TurnK $ProcK // reverse again, but not so important.
                $( $LabN: $CaseN => $TurnN $ProcN )* ;
                $( $LabM: $CaseM => $TurnM $ProcM )+
            }
            $TurnK
        }
        $ProcK
    };

    ( @ENTERING $Ent:expr ;
        $( $LabN:lifetime: $CaseN:pat => $TurnN:block $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $TurnK:block $ProcK:block
    ) => {
        $LabK: {
            match $Ent {
                $CaseK => break $LabK,
              $($CaseN => break $LabN,)*
            }
        }
        $ProcK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::*;

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
            assert_eq!(next_code_point(TEST_VECTOR.split_at(pos).1.as_ref()), ch as u32);
        }
    }

    #[test]
    fn test_memchr_utf8() {
        for ch in TEST_VECTOR.chars() {
            assert_eq!(TEST_VECTOR.memchr(ch).unwrap(), TEST_VECTOR.find(ch).unwrap());
        }
    }

    #[test]
    fn test_memchr() {
        for &byte in TEST_VECTOR.as_bytes() {
            assert_eq!(
                TEST_VECTOR.as_bytes().memchr(byte).unwrap(),
                TEST_VECTOR.as_bytes().iter().position(|b| *b == byte).unwrap(),
            )
        }
    }
}
