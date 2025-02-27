use crate::{common::*, error::*, pattern::*};
use core::marker::PhantomData;
#[cfg(feature = "std")]
use core::{
    mem::{self, MaybeUninit},
    str::from_utf8_unchecked,
};

pub trait SimpleParser2<U>
where
    U: Slice2,
{
    type Captured;

    fn full_match<E: Situation>(&self, slice: U) -> SimpleResult<Self::Captured, E>;

    fn parse<E: Situation>(&self, slice: &mut U) -> SimpleResult<Self::Captured, E>;
}

// impl<U, P> SimpleParser2<U> for P
// where
//     U: Slice2,
//     P: Pattern2<U>,
// {
//     type Captured = P::Captured;

//     #[inline(always)]
//     fn full_match(&self, slice: U) -> Result<Self::Captured, usize> {
//         self.parse(slice).and_then(|(cap, len)| match len == slice.len() {
//             true => Ok(cap),
//             false => Err(len),
//         })
//     }

//     #[inline(always)]
//     fn parse(&self, slice: U) -> Result<(Self::Captured, usize), usize> {
//         let mut state = self.init2();
//         let (t, len) = self
//             .precede2(slice, &mut state, true)
//             .expect("implementation: pull after EOF");

//         if let Transfer::Accepted = t {
//             Ok((self.extract2(slice.split_at(len).0, state), len))
//         } else {
//             Err(len)
//         }
//     }
// }

//==================================================================================================

pub trait Read2 {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize>;
}

impl Read2 for Sliced2 {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        let _ = buf;
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl<R: ::std::io::Read> Read2 for R {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        self.read(buf)
    }
}

//==================================================================================================

/// Uninhabited generic placeholder.
pub enum Sliced2 {}

pub struct Parser2<U, R>(Source2<U, R>)
where
    U: Slice2,
    R: Read2;

enum Source2<U, R>
where
    U: Slice2,
    R: Read2,
{
    Sliced {
        slice: U,
        consumed: usize,
        phantom: PhantomData<R>,
    },

    #[cfg(feature = "std")]
    ReadStr {
        rdr: R,
        eof: bool,
        buf: ::std::vec::Vec<u8>,
        pending: u8,
        consumed: usize,
        discarded: usize,
    },

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

impl<'i> Parser2<&'i str, Sliced2> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'i str) -> Self {
        Self(Source2::Sliced {
            slice: s,
            consumed: 0,
            phantom: PhantomData,
        })
    }

    pub fn from_bstr(bytes: &'i [u8]) -> Option<Self> {
        Some(Self(Source2::Sliced {
            slice: simdutf8::basic::from_utf8(bytes).ok()?,
            consumed: 0,
            phantom: PhantomData,
        }))
    }
}

#[cfg(feature = "std")]
impl<R: Read2> Parser2<&str, R> {
    pub fn from_reader_in_str(reader: R) -> Self {
        Self(Source2::ReadStr {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(0x8000),
            pending: 0,
            consumed: 0,
            discarded: 0,
        })
    }

    pub fn from_reader_in_str_with_capacity(reader: R, capacity: usize) -> Self {
        Self(Source2::ReadStr {
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
impl<R: Read2> Parser2<&[u8], R> {
    pub fn from_reader_in_bytes(reader: R) -> Self {
        Self(Source2::ReadBytes {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(0x8000),
            consumed: 0,
            discarded: 0,
        })
    }

    pub fn from_reader_in_bytes_with_capacity(reader: R, capacity: usize) -> Self {
        Self(Source2::ReadBytes {
            rdr: reader,
            eof: false,
            buf: ::std::vec::Vec::with_capacity(capacity),
            consumed: 0,
            discarded: 0,
        })
    }
}

impl<'i, T> Parser2<&'i [T], Sliced2>
where
    T: Copy + PartialEq,
{
    pub fn from_slice(slice: &'i [T]) -> Self {
        Self(Source2::Sliced {
            slice,
            consumed: 0,
            phantom: PhantomData,
        })
    }
}

//==================================================================================================

impl<'i, R: Read2> Parser2<&'i str, R> {
    pub fn next_str<P, E>(&'i mut self, pat: P) -> ParseResult<P::Captured, E>
    where
        P: Pattern2<&'i str>,
        E: Situation,
    {
        let mut entry = pat.init2();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.precede2::<E>(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ParseError::Mismatched(e)),
                    true => match eof {
                        true => panic!("implementation: pull after EOF"),
                        false => first_time = false,
                    },
                },
            }
        };

        Ok(pat.extract2(self.0.bump_str(len), entry))
    }
}

impl<'i, R: Read2> Source2<&'i str, R> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull_str<E>(&mut self, first_time: bool) -> ParseResult<(&'i str, bool), E>
    where
        E: Situation,
    {
        match self {
            Source2::Sliced { slice, consumed, .. } => {
                let _ = first_time;
                Ok((slice.split_at(*consumed).1, true))
            }

            #[cfg(feature = "std")]
            Source2::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source2::ReadStr {
                rdr,
                eof,
                buf,
                pending,
                consumed,
                discarded,
                ..
            } => {
                // match first_time {
                //     true => Self::buf_first_time(buf, consumed, discarded),
                //     false => Self::buf_subsequent(buf),
                // }

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
                                return Err(ParseError::InvalidUtf8);
                            }
                        } else if let Err(e) = simdutf8::compat::from_utf8(&buf[len_avail - *pending as usize..]) {
                            if e.error_len().is_some() {
                                return Err(ParseError::InvalidUtf8); // IDEA: lossy mode?
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
                    // Safety:
                    // 1. from_utf8_unchecked, because already verified by simdutf8.
                    // 2. Extend lifetime, because `&self` does not outlives `'i`.
                    //   - The returned value can only be used by [`Pattern::precede`];
                    //   - The associated [`Pattern::Internal`] must outlives `'static`.
                    unsafe {
                        mem::transmute::<&str, &'i str>(from_utf8_unchecked(
                            &buf[*consumed..buf.len() - *pending as usize],
                        ))
                    },
                    *eof,
                ))
            }
        }
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn bump_str(&'i mut self, n: usize) -> &'i str {
        match self {
            Source2::Sliced { slice, consumed, .. } => {
                let left = slice[*consumed..]
                    .split_at_checked(n)
                    .expect("implementation: invalid bump")
                    .0;
                *consumed += n;
                left
            }

            #[cfg(feature = "std")]
            Source2::ReadBytes { .. } => unreachable!(),

            #[cfg(feature = "std")]
            Source2::ReadStr { buf, consumed, .. } => {
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
}
