use crate::{common::*, pattern::*};
use core::marker::PhantomData;
#[cfg(feature = "std")]
use core::{
    mem::{self, MaybeUninit},
    str::from_utf8_unchecked,
};

pub mod error;

use self::error::*;

pub trait SimpleParser<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;

    fn full_match(&self, slice: &'i U) -> Result<Self::Captured, usize>;

    fn parse(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize>;
}

impl<'i, U, P> SimpleParser<'i, U> for P
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    type Captured = P::Captured;

    #[inline(always)]
    fn full_match(&self, slice: &'i U) -> Result<Self::Captured, usize> {
        self.parse(slice).and_then(|(cap, len)| match len == slice.len() {
            true => Ok(cap),
            false => Err(len),
        })
    }

    #[inline(always)]
    fn parse(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize> {
        let mut state = self.init();
        let (t, len) = self
            .precede(slice, &mut state, true)
            .expect("implementation: pull after EOF");

        if let Transfer::Accepted = t {
            Ok((self.extract(slice.split_at(len).0, state), len))
        } else {
            Err(len)
        }
    }
}

//==================================================================================================

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> ParseResult<usize>;
}

impl Read for Sliced {
    fn read(&mut self, buf: &mut [u8]) -> ParseResult<usize> {
        let _ = buf;
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl<R: ::std::io::Read> Read for R {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> ParseResult<usize> {
        Ok(self.read(buf)?)
    }
}

//==================================================================================================

/// Uninhabited generic placeholder.
pub enum Sliced {}

pub struct Parser<'src, U, R>(Source<'src, U, R>)
where
    U: ?Sized + Slice,
    R: Read;

enum Source<'src, U, R>
where
    U: ?Sized + Slice,
    R: Read,
{
    Sliced {
        slice: &'src U,
        consumed: usize,
        phantom: PhantomData<R>,
    },

    #[cfg(feature = "std")]
    /// This can only be constructed by [`Parser::from_reader_in_str`], where `U = str`.
    ReadStr {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        pending: u8,
        consumed: usize,
        discarded: usize,
    },

    #[cfg(feature = "std")]
    /// This can only be constructed by [`Parser::from_reader_in_bytes`], where `U = [u8]`,
    /// otherwise UB when calling [`Source::pull`] and [`Source::bump`].
    ReadBytes {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        consumed: usize,
        discarded: usize,
    },
}

//==================================================================================================

impl<'src> Parser<'src, str, Sliced> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'src str) -> Self {
        Self(Source::Sliced {
            slice: s,
            consumed: 0,
            phantom: PhantomData,
        })
    }

    pub fn from_bstr(bytes: &'src [u8]) -> Option<Self> {
        Some(Self(Source::Sliced {
            slice: simdutf8::basic::from_utf8(bytes).ok()?,
            consumed: 0,
            phantom: PhantomData,
        }))
    }
}

#[cfg(feature = "std")]
impl<R: Read> Parser<'_, str, R> {
    pub fn from_reader_in_str(reader: R) -> Self {
        Self(Source::ReadStr {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(0x8000),
            pending: 0,
            consumed: 0,
            discarded: 0,
        })
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
impl<R: Read> Parser<'_, [u8], R> {
    pub fn from_reader_in_bytes(reader: R) -> Self {
        Self(Source::ReadBytes {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(0x8000),
            consumed: 0,
            discarded: 0,
        })
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

impl<'src, T: PartialEq> Parser<'src, [T], Sliced> {
    pub fn from_slice(slice: &'src [T]) -> Self {
        Self(Source::Sliced {
            slice,
            consumed: 0,
            phantom: PhantomData,
        })
    }
}

impl<U: ?Sized + Slice, R: Read> Parser<'_, U, R> {
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
            Source::ReadStr { eof, buf, consumed, .. } | Source::ReadBytes { eof, consumed, buf, .. } => {
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

impl<R: Read> Parser<'_, str, R> {
    pub fn next_str<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                None => match eof {
                    true => panic!("implementation: pull after EOF"),
                    false => first_time = false,
                },
                Some((t, len)) => match t.is_accepted() {
                    true => break len,
                    false => return Error::raise(ErrorKind::Mismatched),
                },
            }
        };

        Ok(pat.extract(self.0.bump_str(len), entry))
    }

    pub fn peek_str<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                None => match eof {
                    true => panic!("implementation: pull after EOF"),
                    false => first_time = false,
                },
                Some((t, len)) => match t.is_accepted() {
                    true => break len,
                    false => return Error::raise(ErrorKind::Mismatched),
                },
            }
        };

        Ok(pat.extract(self.0.slice_str(len), entry))
    }
}

impl<R: Read> Source<'_, str, R> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull_str(&mut self, first_time: bool) -> ParseResult<(&str, bool)> {
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
                                return Error::raise(ErrorKind::InvalidUtf8);
                            }
                        } else if let Err(e) = simdutf8::compat::from_utf8(&buf[len_avail - *pending as usize..]) {
                            if e.error_len().is_some() {
                                return Error::raise(ErrorKind::InvalidUtf8); // IDEA: lossy mode?
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
                from_utf8_unchecked(&buf[*consumed..])
                    .split_at_checked(n)
                    .expect("implementation: invalid slice")
                    .0
            },
        }
    }
}

//==================================================================================================

impl<T: PartialEq, R: Read> Parser<'_, [T], R> {
    #[allow(clippy::should_implement_trait)]
    pub fn next<'i, P: Pattern<'i, [T]>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                None => match eof {
                    true => panic!("implementation: pull after EOF"),
                    false => first_time = false,
                },
                Some((t, len)) => match t.is_accepted() {
                    true => break len,
                    false => return Error::raise(ErrorKind::Mismatched),
                },
            }
        };

        Ok(pat.extract(self.0.bump(len), entry))
    }

    pub fn peek<'i, P: Pattern<'i, [T]>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                None => match eof {
                    true => panic!("implementation: pull after EOF"),
                    false => first_time = false,
                },
                Some((t, len)) => match t.is_accepted() {
                    true => break len,
                    false => return Error::raise(ErrorKind::Mismatched),
                },
            }
        };

        Ok(pat.extract(self.0.slice(len), entry))
    }
}

impl<T: PartialEq, R: Read> Source<'_, [T], R> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull(&mut self, first_time: bool) -> ParseResult<(&[T], bool)> {
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
    use std::prelude::rust_2021::*;

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
            let (s, eof) = src.pull_str(true).unwrap();
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
        let mut par = Parser::from_reader_in_str(buf.as_bytes());
        let mut ctr = 0;
        let mut len = 0;

        par.0.inspect();
        loop {
            ctr += 1;

            match par.next_str(opt(is_ascii..)) {
                Ok(cap) => {
                    if let Some(s) = cap {
                        len += s.len();
                    }
                }
                Err(e) => match e.kind {
                    ErrorKind::Mismatched => (),
                    _ => panic!("{:?}", e),
                },
            }

            match par.next_str(opt(not(is_ascii)..)) {
                Ok(cap) => {
                    if let Some(s) = cap {
                        len += s.len();
                    }
                }
                Err(e) => match e.kind {
                    ErrorKind::Mismatched => (),
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
        let mut par = Parser::from_reader_in_bytes_with_capacity(bytes.as_ref(), 2);

        assert_eq!(par.peek(not(0)..).unwrap(), b"asdf");
        assert_eq!(par.next(not(0)..).unwrap(), b"asdf");
        assert_eq!(par.next([0]).unwrap(), 0);
        assert!(par.exhausted());
    }
}
