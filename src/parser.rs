use crate::{common::*, pattern::*};
use core::{marker::PhantomData, mem, str::from_utf8_unchecked};

pub mod error;

use self::error::*;

pub trait SimpleParser<'i, U>
where
    U: 'i + ?Sized + Slice,
{
    type Captured;

    fn parse_all(&self, slice: &'i U) -> Result<Self::Captured, usize>;

    fn parse(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize>;
}

impl<'i, U, P> SimpleParser<'i, U> for P
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
{
    type Captured = P::Captured;

    #[inline(always)]
    fn parse_all(&self, slice: &'i U) -> Result<Self::Captured, usize> {
        self.parse(slice).and_then(|(cap, len)| match len == slice.len() {
            true => Ok(cap),
            false => Err(slice.len() - len),
        })
    }

    #[inline(always)]
    fn parse(&self, slice: &'i U) -> Result<(Self::Captured, usize), usize> {
        let mut state = self.init();
        let (t, len) = self
            .precede(slice, &mut state, true)
            .expect("implementation error: pull after EOF");

        if let Transfer::Accepted = t {
            Ok((self.extract(slice, state), len))
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
    /// otherwise UB when calling [`Parser::content`].
    ReadBytes {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        consumed: usize,
        discarded: usize,
    },
}

//------------------------------------------------------------------------------

impl<'src> Parser<'src, str, Sliced> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'src str) -> Self {
        todo!()
    }

    pub fn from_bstr(bytes: &'src [u8]) -> Option<Self> {
        todo!()
    }
}

impl<R: Read> Parser<'_, str, R> {
    pub fn from_reader_in_str(reader: R) -> Self {
        todo!()
    }
}

impl<R: Read> Parser<'_, [u8], R> {
    pub fn from_reader_in_bytes(reader: R) -> Self {
        todo!()
    }
}

impl<'src, T: PartialEq> Parser<'src, [T], Sliced> {
    pub fn from_slice(slice: &'src [T]) -> Self {
        todo!()
    }
}

impl<U: ?Sized + Slice, R: Read> Parser<'_, U, R> {
    pub fn into_reader(self) -> Result<R, Self> {
        todo!()
    }
}

//------------------------------------------------------------------------------

impl<R: Read> Parser<'_, str, R> {
    pub fn parse<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        self.0.parse(pat)
    }
}

impl<R: Read> Source<'_, str, R> {
    #[inline(always)]
    fn parse<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.pull(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                None => match eof {
                    true => panic!("implementation error: pull after EOF"),
                    false => first_time = false,
                },
                Some((t, len)) => match t.is_accepted() {
                    true => break len,
                    false => return Error::raise(ErrorKind::Mismatched),
                },
            }
        };

        Ok(pat.extract(self.content_then_bump(len), entry))
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull(&mut self, first_time: bool) -> ParseResult<(&str, bool)> {
        match self {
            Source::Sliced { slice, consumed, .. } => Ok((slice.split_at(*consumed).1, true)),

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
                        let len_delta = rdr.read(unsafe { mem::transmute(buf.spare_capacity_mut()) })?;
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

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn content_then_bump<'i>(&'i mut self, n: usize) -> &'i str {
        match self {
            Source::Sliced { slice, consumed, .. } => {
                let left = slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation error: invalid bump")
                    .0;
                *consumed += n;
                left
            }

            #[cfg(feature = "std")]
            Source::ReadStr { buf, consumed, .. } => {
                let left = unsafe {
                    from_utf8_unchecked(&buf[*consumed..])
                        .split_at_checked(n)
                        .expect("implementation error: invalid bump")
                        .0
                };
                *consumed += n;
                left
            }

            #[cfg(feature = "std")]
            Source::ReadBytes { .. } => unreachable!(),
        }
    }
}

//------------------------------------------------------------------------------

impl<T: PartialEq, R: Read> Parser<'_, [T], R> {
    pub fn parse<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        todo!()
    }
}

impl<T: PartialEq, R: Read> Source<'_, [T], R> {
    fn parse<'i, P: Pattern<'i, str>>(&'i mut self, pat: P) -> ParseResult<P::Captured> {
        todo!()
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn content<'i>(&'i self) -> &'i [T] {
        match &self {
            Source::Sliced { slice, .. } => slice,

            #[cfg(feature = "std")]
            Source::ReadStr { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source::ReadBytes { buf, consumed, .. } => unsafe {
                // SAFETY: See variant doc.
                mem::transmute(&buf[*consumed..])
            },
        }
    }
}

//------------------------------------------------------------------------------

impl<U: ?Sized + Slice, R: Read> Parser<'_, U, R> {}

impl<U: ?Sized + Slice, R: Read> Source<'_, U, R> {
    const INIT_CAP: usize = 32 * 1024;

    pub fn offset(&self) -> usize {
        match self {
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
        match self {
            Source::Sliced { slice, consumed, .. } => *consumed == slice.len(),

            #[cfg(feature = "std")]
            Source::ReadStr { eof, buf, consumed, .. } | Source::ReadBytes { eof, consumed, buf, .. } => {
                *eof && *consumed == buf.len()
            }
        }
    }

    #[inline(always)]
    #[cfg(feature = "std")]
    fn buf_first_time(buf: &mut ::std::vec::Vec<u8>, consumed: &mut usize, discarded: &mut usize) {
        if unlikely(*consumed > p78(buf.len())) {
            buf.drain(..*consumed);
            *discarded += *consumed;
            *consumed = 0;
        }
    }

    #[inline(always)]
    #[cfg(feature = "std")]
    fn buf_subsequent(buf: &mut ::std::vec::Vec<u8>) {
        if unlikely(buf.len() > p88(buf.capacity())) {
            buf.reserve_exact((buf.capacity() / 4).next_power_of_two());
        }
    }
}

const fn p78(n: usize) -> usize {
    n / 2 + n / 4 + n / 32
}

const fn p88(n: usize) -> usize {
    n - n / 8
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
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
        let mut buf = String::with_capacity(1024 * 1024 * 4);
        for i in 0..1024 * 1024u32 {
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
            buf: Vec::with_capacity(8 * 1024),
            pending: 0,
            consumed: 0,
            discarded: 0,
        };

        let mut ctr = 0;
        loop {
            ctr += 1;
            let (s, eof) = src.pull(true).unwrap();
            let len = (random(ctr) as usize % s.len()..)
                .find(|n| s.is_char_boundary(*n))
                .unwrap();

            simdutf8::compat::from_utf8(src.content_then_bump(len).as_bytes()).unwrap();

            if eof {
                break;
            }
        }

        std::println!("{}", ctr);
    }
}
