use crate::{common::*, error::*, line_col::*, pattern::*};
use core::{marker::PhantomData, num::NonZeroUsize};
#[cfg(feature = "std")]
use core::{
    mem::{self, MaybeUninit},
    str::from_utf8_unchecked,
};

pub trait Read {
    fn read<E: Situation>(&mut self, buf: &mut [u8]) -> Result<usize, ProviderError<E>>;
}

impl Read for Sliced {
    fn read<E: Situation>(&mut self, buf: &mut [u8]) -> Result<usize, ProviderError<E>> {
        let _ = buf;
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl<R: ::std::io::Read> Read for R {
    #[inline(always)]
    fn read<E: Situation>(&mut self, buf: &mut [u8]) -> Result<usize, ProviderError<E>> {
        self.read(buf).map_err(Into::into)
    }
}

//==================================================================================================

/// Uninhabited generic placeholder.
pub enum Sliced {}

pub struct Provider<'i, U, R>(Source<'i, U, R>)
where
    U: ?Sized + Slice,
    R: Read;

/*
    Implementation notes:
    This interface cannot provide `reiter` or `joined` functionality.
*/

enum Source<'i, U, R>
where
    U: ?Sized + Slice,
    R: Read,
{
    Sliced {
        slice: &'i U,
        consumed: usize,
        phantom: PhantomData<R>,
    },

    /// This can only be constructed by [`Provider::from_reader_in_str`], where `U = str`.
    #[cfg(feature = "std")]
    ReadStr {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        pending: u8,
        consumed: usize,
        discarded: usize,
    },

    /// This can only be constructed by [`Provider::from_reader_in_bytes`], where `U = [u8]`,
    /// otherwise UB when calling [`Source::pull`] and [`Source::bump`].
    #[cfg(feature = "std")]
    ReadBytes {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        consumed: usize,
        discarded: usize,
    },
}

//==================================================================================================

impl<'i> Provider<'i, str, Sliced> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'i str) -> Self {
        Self(Source::Sliced {
            slice: s,
            consumed: 0,
            phantom: PhantomData,
        })
    }

    pub fn from_bstr(bytes: &'i [u8]) -> Option<Self> {
        Some(Self(Source::Sliced {
            slice: simdutf8::basic::from_utf8(bytes).ok()?,
            consumed: 0,
            phantom: PhantomData,
        }))
    }
}

#[cfg(feature = "std")]
impl<R: Read> Provider<'_, str, R> {
    pub fn from_reader_in_str(reader: R) -> Self {
        Self::from_reader_in_str_with_capacity(reader, 0x8000)
    }

    pub fn from_reader_in_str_with_capacity(reader: R, capacity: usize) -> Self {
        Self(Source::ReadStr {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(capacity),
            pending: 0,
            consumed: 0,
            discarded: 0,
        })
    }
}

#[cfg(feature = "std")]
impl<R: Read> Provider<'_, [u8], R> {
    pub fn from_reader_in_bytes(reader: R) -> Self {
        Self::from_reader_in_bytes_with_capacity(reader, 0x8000)
    }

    pub fn from_reader_in_bytes_with_capacity(reader: R, capacity: usize) -> Self {
        Self(Source::ReadBytes {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(capacity),
            consumed: 0,
            discarded: 0,
        })
    }
}

impl<'i, T> Provider<'i, [T], Sliced>
where
    T: Copy + PartialEq,
{
    pub fn from_slice(slice: &'i [T]) -> Self {
        Self(Source::Sliced {
            slice,
            consumed: 0,
            phantom: PhantomData,
        })
    }
}

impl<U, R> Provider<'_, U, R>
where
    U: ?Sized + Slice,
    R: Read,
{
    pub fn offset(&self) -> usize {
        match &self.0 {
            Source::Sliced { consumed, .. } => *consumed,

            #[cfg(feature = "std")]
            Source::ReadStr {
                consumed, discarded, ..
            }
            | Source::ReadBytes {
                consumed, discarded, ..
            } => discarded + *consumed,
        }
    }

    pub fn exhausted(&self) -> bool {
        match &self.0 {
            Source::Sliced { slice, consumed, .. } => *consumed == slice.len(),

            #[cfg(feature = "std")]
            Source::ReadStr { eof, buf, consumed, .. } | Source::ReadBytes { eof, buf, consumed, .. } => {
                *eof && *consumed == buf.len()
            }
        }
    }

    pub fn into_reader(self) -> Result<R, Self> {
        match self.0 {
            Source::Sliced { .. } => Err(self),

            #[cfg(feature = "std")]
            Source::ReadStr { rdr, .. } | Source::ReadBytes { rdr, .. } => Ok(rdr),
        }
    }
}

//==================================================================================================

impl<R: Read> Provider<'_, str, R> {
    pub fn next_str<'j, P, E>(&'j mut self, pat: &P) -> ProviderResult<P::Captured, E>
    where
        P: Pattern<'j, str, E>,
        E: Situation,
    {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.advance(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ProviderError::Mismatched(e)),
                    true => match eof {
                        true => panic!("implementation: pull after EOF"),
                        false => first_time = false,
                    },
                },
            }
        };

        Ok(pat.extract(self.0.bump_str(len), entry))
    }

    pub fn peek_str<'j, P, E>(&'j mut self, pat: &P) -> ProviderResult<P::Captured, E>
    where
        P: Pattern<'j, str, E>,
        E: Situation,
    {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.advance(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ProviderError::Mismatched(e)),
                    true => match eof {
                        true => panic!("implementation: pull after EOF"),
                        false => first_time = false,
                    },
                },
            }
        };

        Ok(pat.extract(self.0.slice_str(len), entry))
    }

    #[allow(unsafe_code)]
    pub fn content(&self) -> (usize, &str, &str) {
        match &self.0 {
            Source::Sliced { slice, consumed, .. } => {
                let (left, right) = slice.split_at(*consumed);
                (0, left, right)
            }

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadStr {
                buf,
                pending,
                consumed,
                discarded,
                ..
            } => {
                let (left, right) =
                    unsafe { from_utf8_unchecked(&buf[..buf.len() - *pending as usize]) }.split_at(*consumed);
                (*discarded, left, right)
            }
        }
    }

    pub fn line_col(&self) -> Option<(NonZeroUsize, NonZeroUsize)> {
        let (discarded, consumed, _) = self.content();
        if discarded != 0 {
            return None;
        }

        line_col(consumed, consumed.len())
    }

    pub fn line_col_span(&self, length: usize) -> Option<((NonZeroUsize, NonZeroUsize), (NonZeroUsize, NonZeroUsize))> {
        let (discarded, consumed, avail) = self.content();
        if discarded != 0 {
            return None;
        }

        let loc2 = line_col(avail, length)?;
        let loc1 = line_col(consumed, consumed.len())?;

        Some((loc1, series_locate(loc1, loc2)))
    }
}

impl<R: Read> Source<'_, str, R> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull_str<E: Situation>(&mut self, first_time: bool) -> ProviderResult<(&str, bool), E> {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                let _ = first_time;
                Ok((slice.split_at(*consumed).1, true))
            }

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadStr {
                rdr,
                eof,
                buf,
                pending,
                consumed,
                discarded,
                ..
            } => {
                match first_time {
                    true => Self::buf_first_time(buf, consumed, discarded),
                    false => Self::buf_subsequent(buf),
                }

                if buf.len() < buf.capacity() {
                    loop {
                        let len_avail = buf.len();
                        let len_delta = rdr.read(unsafe {
                            mem::transmute::<&mut [MaybeUninit<u8>], &mut [u8]>(buf.spare_capacity_mut())
                        })?;
                        unsafe { buf.set_len(len_avail + len_delta) };

                        *eof = len_delta == 0;

                        if *eof {
                            if *pending != 0 {
                                return Err(ProviderError::InvalidUtf8);
                            }
                        } else if let Err(e) = simdutf8::compat::from_utf8(&buf[len_avail - *pending as usize..]) {
                            if e.error_len().is_some() {
                                return Err(ProviderError::InvalidUtf8); // IDEA: lossy mode?
                            } else {
                                match e.valid_up_to() {
                                    0 => continue,
                                    n => *pending = (*pending as usize + len_delta - n) as u8,
                                }
                            }
                        } else {
                            *pending = 0
                        }

                        break;
                    }
                }

                Ok((
                    // Safety: already verified by simdutf8.
                    unsafe { from_utf8_unchecked(&buf[*consumed..buf.len() - *pending as usize]) },
                    *eof,
                ))
            }
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn bump_str(&mut self, n: usize) -> &str {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                let left = slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation: invalid bump")
                    .0;
                *consumed += n;
                left
            }

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadStr { buf, consumed, .. } => {
                let left = unsafe {
                    // Safety: already verified by simdutf8.
                    from_utf8_unchecked(&buf[*consumed..])
                        .split_at_checked(n)
                        .expect("implementation: invalid bump")
                        .0
                };
                *consumed += n;
                left
            }
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn slice_str(&self, n: usize) -> &str {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation: invalid slice")
                    .0
            }

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadStr { buf, consumed, .. } => unsafe {
                // Safety: already verified by simdutf8.
                from_utf8_unchecked(&buf[*consumed..])
                    .split_at_checked(n)
                    .expect("implementation: invalid slice")
                    .0
            },
        }
    }
}

//==================================================================================================

impl<T, R> Provider<'_, [T], R>
where
    T: Copy + PartialEq,
    R: Read,
{
    #[allow(clippy::should_implement_trait)]
    pub fn next<'j, P, E>(&'j mut self, pat: &P) -> ProviderResult<P::Captured, E>
    where
        P: Pattern<'j, [T], E>,
        E: Situation,
    {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull(first_time)?;
            match pat.advance(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ProviderError::Mismatched(e)),
                    true => match eof {
                        true => panic!("implementation: pull after EOF"),
                        false => first_time = false,
                    },
                },
            }
        };

        Ok(pat.extract(self.0.bump(len), entry))
    }

    pub fn peek<'j, P, E>(&'j mut self, pat: &P) -> ProviderResult<P::Captured, E>
    where
        P: Pattern<'j, [T], E>,
        E: Situation,
    {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull(first_time)?;
            match pat.advance(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ProviderError::Mismatched(e)),
                    true => match eof {
                        true => panic!("implementation: pull after EOF"),
                        false => first_time = false,
                    },
                },
            }
        };

        Ok(pat.extract(self.0.slice(len), entry))
    }

    #[allow(unsafe_code)]
    pub fn content(&self) -> (usize, &[T], &[T]) {
        match &self.0 {
            Source::Sliced { slice, consumed, .. } => {
                let (left, right) = slice.split_at(*consumed);
                (0, left, right)
            }

            #[cfg(feature = "std")]
            Source::ReadStr { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadBytes {
                buf,
                consumed,
                discarded,
                ..
            } => {
                let (left, right) = unsafe { mem::transmute::<&[u8], &[T]>(buf.as_ref()) }.split_at(*consumed);
                (*discarded, left, right)
            }
        }
    }
}

impl<T, R> Source<'_, [T], R>
where
    T: Copy + PartialEq,
    R: Read,
{
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull<E: Situation>(&mut self, first_time: bool) -> ProviderResult<(&[T], bool), E> {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                let _ = first_time;
                Ok((slice.split_at(*consumed).1, true))
            }

            #[cfg(feature = "std")]
            Source::ReadStr { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadBytes {
                rdr,
                eof,
                buf,
                consumed,
                discarded,
                ..
            } => {
                match first_time {
                    true => Self::buf_first_time(buf, consumed, discarded),
                    false => Self::buf_subsequent(buf),
                }

                if buf.len() < buf.capacity() {
                    let len_avail = buf.len();
                    let len_delta = rdr.read(unsafe {
                        mem::transmute::<&mut [MaybeUninit<u8>], &mut [u8]>(buf.spare_capacity_mut())
                    })?;
                    unsafe { buf.set_len(len_avail + len_delta) };

                    *eof = len_delta == 0;
                }

                Ok((
                    // Safety: See variant doc.
                    unsafe { mem::transmute::<&[u8], &[T]>(&buf[*consumed..]) },
                    *eof,
                ))
            }
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn bump(&mut self, n: usize) -> &[T] {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                let left = slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation: invalid bump")
                    .0;
                *consumed += n;
                left
            }

            #[cfg(feature = "std")]
            Source::ReadStr { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadBytes { buf, consumed, .. } => {
                let left = unsafe {
                    // Safety: See variant doc.
                    mem::transmute::<&[u8], &[T]>(
                        buf[*consumed..]
                            .split_at_checked(n)
                            .expect("implementation: invalid bump")
                            .0,
                    )
                };
                *consumed += n;
                left
            }
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn slice(&self, n: usize) -> &[T] {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation: invalid slice")
                    .0
            }

            #[cfg(feature = "std")]
            Source::ReadStr { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadBytes { buf, consumed, .. } => unsafe {
                // Safety: See variant doc.
                mem::transmute::<&[u8], &[T]>(
                    buf[*consumed..]
                        .split_at_checked(n)
                        .expect("implementation: invalid slice")
                        .0,
                )
            },
        }
    }
}

//==================================================================================================

#[cfg(feature = "std")]
impl<U: ?Sized + Slice, R: Read> Source<'_, U, R> {
    #[inline(always)]
    fn buf_first_time(buf: &mut ::std::vec::Vec<u8>, consumed: &mut usize, discarded: &mut usize) {
        const fn p75(n: usize) -> usize {
            n / 2 + n / 4
        }
        // Should be `>`, keep branch prediction when `buf.len() == 0`.
        if unlikely(*consumed > p75(buf.len())) {
            buf.drain(..*consumed);
            *discarded += *consumed;
            *consumed = 0;
        }
    }

    #[inline(always)]
    fn buf_subsequent(buf: &mut ::std::vec::Vec<u8>) {
        const fn p88(n: usize) -> usize {
            n - n / 8
        }
        // Must be `>=`, otherwise infinite loop when `buf.len() == 0`.
        if unlikely(buf.len() >= p88(buf.capacity())) {
            buf.reserve_exact((buf.capacity() / 4) | 0x2000);
        }
    }
}

#[cfg(all(test, feature = "std"))]
impl<U: ?Sized + Slice, R: Read> Source<'_, U, R> {
    fn inspect(&self) {
        match self {
            Source::Sliced { .. } => (),

            Source::ReadStr {
                buf,
                consumed,
                discarded,
                ..
            }
            | Source::ReadBytes {
                buf,
                consumed,
                discarded,
                ..
            } => println!(
                "{:>8} ({:4.2} consumed) | CAP: {:>8} | DISCARD: {:>8}",
                buf.len(),
                *consumed as f32 / buf.len() as f32,
                buf.capacity(),
                discarded
            ),
        }
    }
}

//==================================================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::prelude::*;
    use std::prelude::rust_2024::*;

    fn random(mut x: u32) -> u32 {
        x ^= x >> 16;
        x = x.wrapping_mul(0x21f0aaad);
        x ^= x >> 15;
        x = x.wrapping_mul(0xd35a2d97);
        x ^= x >> 16;
        x
    }

    fn random_string(seed: u32) -> String {
        let mut buf = String::with_capacity(0x100_000 * 4);
        for i in 0..0x100_000 {
            buf.push(char::from_u32(random(i ^ seed) % 0x110000).unwrap_or('\u{fffd}'));
        }
        buf
    }

    #[test]
    fn test_pull_str() {
        let buf = random_string(1123);
        let mut src = Source::ReadStr {
            rdr: buf.as_bytes(),
            eof: false,
            buf: Vec::with_capacity(0x8000),
            pending: 0,
            consumed: 0,
            discarded: 0,
        };

        let mut ctr = 0;
        src.inspect();
        loop {
            ctr += 1;
            let (s, eof) = src.pull_str::<SimpleError>(true).unwrap();
            let len = (random(ctr) as usize % s.len()..)
                .find(|n| s.is_char_boundary(*n))
                .unwrap();

            simdutf8::compat::from_utf8(src.bump_str(len).as_bytes()).unwrap();

            src.inspect();
            if eof {
                break;
            }
        }

        println!("{}", ctr);
    }

    #[test]
    fn test_parse_str() {
        let buf = random_string(42);
        let mut par = Provider::from_reader_in_str(buf.as_bytes());
        let mut ctr = 0;
        let mut len = 0;

        par.0.inspect();
        loop {
            ctr += 1;

            match par.next_str::<_, SimpleError>(&opt(is_ascii..)) {
                Ok(cap) => {
                    if let Some(s) = cap {
                        len += s.len();
                    }
                }
                Err(e) => match e {
                    ProviderError::Mismatched(_) => (),
                    _ => panic!("{:?}", e),
                },
            }

            match par.next_str::<_, SimpleError>(&opt(not(is_ascii)..)) {
                Ok(cap) => {
                    if let Some(s) = cap {
                        len += s.len();
                    }
                }
                Err(e) => match e {
                    ProviderError::Mismatched(_) => (),
                    _ => panic!("{:?}", e),
                },
            }

            par.0.inspect();
            if par.exhausted() {
                break;
            }
        }

        println!("{}", ctr);

        assert_eq!(len, buf.len());
    }

    #[test]
    fn test_parse_bytes() {
        let bytes = b"asdf\0";
        let mut par = Provider::from_reader_in_bytes_with_capacity(bytes.as_ref(), 2);

        assert_eq!(par.peek::<_, SimpleError>(&(not(0)..)).unwrap(), b"asdf");
        assert_eq!(par.next::<_, SimpleError>(&(not(0)..)).unwrap(), b"asdf");
        assert_eq!(par.next::<_, SimpleError>(&[0]).unwrap(), 0);
        assert!(par.exhausted());
    }
}
