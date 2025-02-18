use crate::{common::*, pattern::*};
use core::{marker::PhantomData, mem, str::from_utf8_unchecked};
use std::vec::Vec;

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

pub struct Parser<'src, U, R>
where
    U: ?Sized + Slice,
    R: Read,
{
    src: Source<'src, U, R>,
    consumed: usize,
}

enum Source<'src, U, R>
where
    U: ?Sized + Slice,
    R: Read,
{
    Sliced {
        slice: &'src U,
        phantom: PhantomData<R>,
    },

    #[cfg(feature = "std")]
    ReadStr {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        pending: u8,
        discarded: usize,
    },

    #[cfg(feature = "std")]
    ReadBytes {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
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
        let mut first_time = false;
        let mut entry = pat.init();
        loop {
            let (slice, eof) = self.src.pull(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
                Ok((t, len)) => {
                    if let Transfer::Accepted = t {
                        debug_assert!(slice.is_char_boundary(len), "implementation error: invalid bump");
                        self.consumed += len;

                        return Ok(pat.extract(slice, entry));
                    } else {
                        return Error::raise(ErrorKind::Mismatched);
                    }
                }

                Err(_) => todo!(),
            }

            first_time = true;
        }
    }
}

impl<R: Read> Source<'_, str, R> {
    #[allow(unsafe_code)]
    fn pull<'i>(&'i mut self, first_time: bool) -> ParseResult<(&'i str, bool)> {
        match first_time {
            true => self.rearrange_buffer(),
            false => self.reserve_buffer(),
        }

        todo!()
        //         match self {
        //             Parser::Sliced {
        //                 slice, off_consumed, ..
        //             } => return Ok((slice.split_at(*off_consumed).1, true)),
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadBytes { .. } => unreachable!(),
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadStr {
        //                 rdr,
        //                 eof,
        //                 buf,
        //                 pending,
        //                 off_consumed,
        //                 ..
        //             } => {
        //                 loop {
        //                     let len_avail = buf.len();
        //                     let len_delta = rdr.read(unsafe { mem::transmute(buf.spare_capacity_mut()) })?;
        //                     unsafe { buf.set_len(len_avail + len_delta) };
        //
        //                     *eof = len_delta == 0;
        //
        //                     if *eof {
        //                         if *pending != 0 {
        //                             return Error::raise(ErrorKind::InvalidUtf8);
        //                         }
        //                     } else if let Err(e) = simdutf8::compat::from_utf8(&buf[len_avail - *pending as usize..]) {
        //                         if e.error_len().is_some() {
        //                             return Error::raise(ErrorKind::InvalidUtf8); // IDEA: lossy mode?
        //                         } else {
        //                             match e.valid_up_to() {
        //                                 0 => continue,
        //                                 n => *pending = (*pending as usize + len_delta - n) as u8,
        //                             }
        //                         }
        //                     } else {
        //                         *pending = 0
        //                     }
        //
        //                     break;
        //                 }
        //
        //                 return unsafe {
        //                     Ok((
        //                         from_utf8_unchecked(&buf[*off_consumed..buf.len() - *pending as usize]),
        //                         *eof,
        //                     ))
        //                 };
        //             }
        //         }
    }
}

//------------------------------------------------------------------------------

impl<T: PartialEq, R: Read> Parser<'_, [T], R> {
    pub fn parse<'i>(&'i mut self, pat: impl Pattern<'i, [T]>) {}
}

//------------------------------------------------------------------------------

impl<U: ?Sized + Slice, R: Read> Source<'_, U, R> {
    const INIT_CAP: usize = 32 * 1024;

    fn consumed(&self) -> usize {
        todo!()
        //         match self {
        //             Parser::Sliced { off_consumed, .. } => *off_consumed,
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadStr {
        //                 off_consumed,
        //                 did_consumed,
        //                 ..
        //             }
        //             | Parser::ReadBytes {
        //                 off_consumed,
        //                 did_consumed,
        //                 ..
        //             } => *did_consumed + *off_consumed,
        //         }
    }

    fn exhausted(&self) -> bool {
        todo!()
        //         match self {
        //             Parser::Sliced {
        //                 slice, off_consumed, ..
        //             } => *off_consumed == slice.len(),
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadStr {
        //                 eof, buf, off_consumed, ..
        //             }
        //             | Parser::ReadBytes {
        //                 eof, buf, off_consumed, ..
        //             } => *eof && *off_consumed == buf.len(),
        //         }
    }

    #[inline(always)]
    fn rearrange_buffer(&mut self) {
        todo!()
        //         match self {
        //             Parser::Sliced { .. } => (),
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadStr {
        //                 buf,
        //                 off_consumed,
        //                 did_consumed,
        //                 ..
        //             }
        //             | Parser::ReadBytes {
        //                 buf,
        //                 off_consumed,
        //                 did_consumed,
        //                 ..
        //             } => {
        //                 if unlikely(*off_consumed > p78(buf.capacity())) {
        //                     buf.drain(..*off_consumed);
        //                     *did_consumed += *off_consumed;
        //                     *off_consumed = 0
        //                 }
        //             }
        //         }
    }

    #[inline(always)]
    fn reserve_buffer(&mut self) {
        todo!()
        //         match self {
        //             Parser::Sliced { .. } => (),
        //
        //             #[cfg(feature = "std")]
        //             Parser::ReadStr { buf, off_consumed, .. } | Parser::ReadBytes { buf, off_consumed, .. } => {
        //                 if unlikely(*off_consumed > p87(buf.capacity())) {
        //                     buf.reserve((buf.capacity() / 2).next_power_of_two());
        //                 }
        //             }
        //         }
    }
}

const fn p78(n: usize) -> usize {
    n / 2 + n / 4 + n / 32
}

const fn p87(n: usize) -> usize {
    n - n / 8
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::string::String;

    fn random(mut x: u32) -> u32 {
        x = x.wrapping_add(0x16a09e66);
        x ^= x >> 17;
        x = x.wrapping_mul(0x1bb67ae8);
        x ^= x >> 17;
        x = x.wrapping_mul(0x23c6ef37);
        x ^= x >> 17;
        x = x.wrapping_mul(0x2a54ff53);
        x ^= x >> 17;
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
        todo!()
        // for i in 0..10 {
        //     let buf = random_string(i * 1123);
        //     let mut par = Parser::from_reader_in_str(buf.as_bytes());
        //     loop {
        //         let (s, eof) = par.pull(true).unwrap();
        //         assert!(simdutf8::basic::from_utf8(s.as_bytes()).is_ok());
        //         if eof {
        //             break;
        //         }
        //     }
        // }
    }
}
