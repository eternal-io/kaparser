use crate::{common::*, error::*, pattern::*};
use core::marker::PhantomData;
#[cfg(feature = "std")]
use core::{
    mem::{self, MaybeUninit},
    str::from_utf8_unchecked,
};

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

pub struct Provider<U, R>(Source2<U, R>)
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

impl<'i> Provider<&'i str, Sliced2> {
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
impl<R: Read2> Provider<&str, R> {
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
impl<R: Read2> Provider<&[u8], R> {
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

impl<'i, T> Provider<&'i [T], Sliced2>
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

impl<'i, R: Read2> Provider<&'i str, R> {
    pub fn next_str<P, E>(&'i mut self, pat: &P) -> ProvideResult<P::Captured, E>
    where
        P: Pattern2<&'i str, E>,
        E: Situation,
    {
        let mut entry = pat.init2();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.precede2(slice, &mut entry, eof) {
                Ok(len) => break len,
                Err(e) => match e.is_unfulfilled() {
                    false => return Err(ProvideError::Mismatched(e)),
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
    fn pull_str<E>(&mut self, first_time: bool) -> ProvideResult<(&'i str, bool), E>
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
                                return Err(ProvideError::InvalidUtf8);
                            }
                        } else if let Err(e) = simdutf8::compat::from_utf8(&buf[len_avail - *pending as usize..]) {
                            if e.error_len().is_some() {
                                return Err(ProvideError::InvalidUtf8); // IDEA: lossy mode?
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
