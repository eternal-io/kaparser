use crate::{common::*, error::*, pattern::*};
use core::marker::PhantomData;
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
        Ok(self.read(buf)?)
    }
}

//==================================================================================================

/// Uninhabited generic placeholder.
pub enum Sliced {}

pub struct Provider<U, R>(Source<U, R>)
where
    U: Slice,
    R: Read;

enum Source<U, R>
where
    U: Slice,
    R: Read,
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

impl<'i> Provider<&'i str, Sliced> {
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
impl<R: Read> Provider<&str, R> {
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
impl<R: Read> Provider<&[u8], R> {
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

impl<'i, T> Provider<&'i [T], Sliced>
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

//==================================================================================================

impl<'i, R: Read> Provider<&'i str, R> {
    pub fn next_str<P, E>(&'i mut self, pat: &P) -> ProviderResult<P::Captured, E>
    where
        P: Pattern<&'i str, E>,
        E: Situation,
    {
        let mut entry = pat.init();
        let mut first_time = true;
        let len = loop {
            let (slice, eof) = self.0.pull_str(first_time)?;
            match pat.precede(slice, &mut entry, eof) {
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
}

impl<'i, R: Read> Source<&'i str, R> {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn pull_str<E>(&mut self, first_time: bool) -> ProviderResult<(&'i str, bool), E>
    where
        E: Situation,
    {
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
}
