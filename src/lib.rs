#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use core::{
    error::Error,
    fmt, mem,
    num::NonZeroU16,
    ops::Range,
    ptr::{addr_of_mut, copy_nonoverlapping},
    str::from_utf8_unchecked,
};

/// Re-exported [paste](https://docs.rs/paste) macro,
/// because [`token_set!`] needs concat identifiers.
pub use paste::paste;

mod common;

pub use common::*;

#[cfg(test)]
mod tests;

pub type TheResult<T> = Result<T, Box<dyn Error>>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> TheResult<usize>;
}

pub trait Situate {
    fn situate(&mut self, to: (NonZeroU16, NonZeroU16), from: Option<(NonZeroU16, NonZeroU16)>);
}

#[derive(Debug)]
pub struct Utf8Error;

pub struct Utf8Parser<'source, R: Read> {
    src: Source<'source, R>,
    eof: bool,

    locking: bool,
    /* It's decided not to provide nested-select functionality,
     * because kaparser just works at the lowest level. */
    selecting: bool,
    /** The original `off_consumed` since [`Self::select_begin`],
     ** so always `off_selected <= off_consumed`. */
    off_selected: usize,
    off_consumed: usize,
    did_consumed: usize,

    ctr_line_selected: u16,
    ctr_line_consumed: u16,
    /* Column count in characters, not bytes. Only `\n` increases the line count.
     * All other characters increase the column count by one, includes `\t`, `\r`, `\0` etc. */
    ctr_column_selected: u16,
    ctr_column_consumed: u16,

    lookahead: Peeked,
}

/// Uninhabited generic placeholder.
pub enum Slice {}

enum Source<'source, R: Read> {
    Borrowed {
        slice: &'source str,
    },
    Reader {
        rdr: R,
        buf: Box<[u8]>,
        buf_cap: usize,
        off_read: usize,
        off_valid: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Peeked {
    None,
    Len1 = 1,
    Len2 = 2,
    Len3 = 3,
    Len4 = 4,
    Newline,
}

//==================================================================================================

impl Read for Slice {
    fn read(&mut self, buf: &mut [u8]) -> TheResult<usize> {
        let _ = buf;
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl<R: std::io::Read> Read for R {
    fn read(&mut self, buf: &mut [u8]) -> TheResult<usize> {
        match self.read(buf) {
            Ok(n) => Ok(n),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Error for Utf8Error {}

impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid UTF-8 bytes")
    }
}

impl From<char> for Peeked {
    fn from(ch: char) -> Self {
        match ch == '\n' {
            true => Self::Newline,
            false => match ch.len_utf8() {
                1 => Self::Len1,
                2 => Self::Len2,
                3 => Self::Len3,
                4 => Self::Len4,
                _ => unreachable!(),
            },
        }
    }
}

impl<'source> Utf8Parser<'source, Slice> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(slice: &'source str) -> Self {
        Self {
            src: Source::Borrowed { slice },
            eof: true,

            locking: false,
            selecting: false,
            off_selected: 0,
            off_consumed: 0,
            did_consumed: 0,

            ctr_line_selected: 0,
            ctr_line_consumed: 0,
            ctr_column_selected: 0,
            ctr_column_consumed: 0,

            lookahead: Peeked::None,
        }
    }

    pub fn from_bytes(bytes: &'source [u8]) -> Result<Self, Utf8Error> {
        simdutf8::basic::from_utf8(bytes)
            .map(Self::from_str)
            .map_err(|_| Utf8Error)
    }
}

impl<R: Read> Utf8Parser<'static, R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            src: Source::Reader {
                rdr: reader,
                buf: unsafe { Box::new_uninit_slice(Self::INIT_CAP).assume_init() },
                buf_cap: Self::INIT_CAP,
                off_read: 0,
                off_valid: 0,
            },
            eof: false,

            locking: false,
            selecting: false,
            off_selected: 0,
            off_consumed: 0,
            did_consumed: 0,

            ctr_line_selected: 0,
            ctr_line_consumed: 0,
            ctr_column_selected: 0,
            ctr_column_consumed: 0,

            lookahead: Peeked::None,
        }
    }
}

impl<R: Read> Utf8Parser<'_, R> {
    const INIT_CAP: usize = 32 * 1024;
    const THRESHOLD: usize = 8 * 1024;

    /// Bumps the offset of the [`content`](Self::content) to parse.
    ///
    /// # Panics
    ///
    /// Panics if the `n`th byte is not the first byte of UTF-8 code point sequence,
    /// and `n` is not equal to the content length.
    pub fn bump(&mut self, n: usize) {
        if !self.content().is_char_boundary(n) {
            panic!("{} is not at a UTF-8 character boundary", n)
        }

        let line = addr_of_mut!(self.ctr_line_consumed);
        let column = addr_of_mut!(self.ctr_column_consumed);
        self.content_behind(0..n).chars().for_each(|ch| unsafe {
            if ch == '\n' {
                *line = (*line).saturating_add(1);
                *column = 0;
            } else {
                *column += (*column).saturating_add(1);
            }
        });

        self.off_consumed += n;
    }

    /// Returns the unconsumed, buffered input.
    #[inline]
    pub fn content(&self) -> &str {
        unsafe {
            match &self.src {
                Source::Borrowed { slice } => slice.get_unchecked(self.off_consumed..),
                Source::Reader { buf, off_valid, .. } => {
                    from_utf8_unchecked(buf.get_unchecked(self.off_consumed..*off_valid))
                }
            }
        }
    }

    #[inline]
    fn content_behind(&self, span: Range<usize>) -> &str {
        unsafe {
            match &self.src {
                Source::Borrowed { slice } => slice.get_unchecked(span),
                Source::Reader { buf, .. } => from_utf8_unchecked(buf.get_unchecked(span)),
            }
        }
    }

    /// Returns the count of totally consumed bytes.
    pub fn consumed(&self) -> usize {
        self.did_consumed + self.off_consumed
    }

    /// Returns `true` if encountered the EOF and all the buffered input is consumed.
    pub fn exhausted(&self) -> bool {
        match &self.src {
            Source::Borrowed { slice } => self.off_consumed == slice.len(),
            Source::Reader { off_valid, .. } => self.eof && self.off_consumed == *off_valid,
        }
    }

    pub fn situate<S>(&self, situation: &mut S)
    where
        S: Situate,
    {
        let to = (
            NonZeroU16::try_from(self.ctr_line_consumed.saturating_add(1)).unwrap(),
            NonZeroU16::try_from(self.ctr_column_consumed.saturating_add(1)).unwrap(),
        );
        let from = self.selecting.then(|| {
            (
                NonZeroU16::try_from(self.ctr_line_selected.saturating_add(1)).unwrap(),
                NonZeroU16::try_from(self.ctr_column_selected.saturating_add(1)).unwrap(),
            )
        });

        situation.situate(to, from);
    }

    //------------------------------------------------------------------------------

    /// TODO! Pull bytes, 拥有不同的行为当：
    /// - 无选区时，只有少于当 content 少于 8 KiB 时才会继续拉取内容。
    /// - 有选区时，总是拉取更多内容，并且每次调用至多会使缓冲区扩大一倍。
    ///
    /// 通常不需要手动调用这个方法，其它方法都会自动拉取内容。
    pub fn pull(&mut self) -> TheResult<()> {
        loop {
            let Source::Reader {
                rdr,
                buf,
                buf_cap,
                off_read,
                off_valid,
            } = &mut self.src
            else {
                return Ok(());
            };

            if self.locking || self.selecting {
                /* Pull without deprecate previous bytes */

                const fn m7d8(n: usize) -> usize {
                    (n >> 1) + (n >> 2) + (n >> 3)
                }

                if *off_read > m7d8(*buf_cap) {
                    *buf_cap <<= 1;
                    if *buf_cap > buf.len() {
                        unsafe {
                            let mut buf_new = Box::new_uninit_slice(*buf_cap).assume_init();
                            copy_nonoverlapping(buf.as_ptr(), buf_new.as_mut_ptr(), *off_read);
                            drop(mem::replace(buf, buf_new));
                        }
                    }
                }
            } else {
                /* Pull with allow deprecate previous bytes */

                /**************************************************************************************************
                 *            THRES_REARRANGE       INIT_CAP                                                      *
                 *                          ↓       ↓                                                             *
                 *  +-------+-------+-------+-------+                                                             *
                 *  |       |       |       |<<<<<<<| buffer                                                      *
                 *  +-------+-------+-------+-------+                                                             *
                 *                          '                                                                     *
                 *                         ^~~~~~~~~$ Next first `if` captured (`content().len() > THRES`),       *
                 *                          '         the span can be shifted/expanded arbitrary.                 *
                 *                          '                                                                     *
                 *                     ^~~~~'~~$      Next second `if` captured (`content().len() <= THRES`),     *
                 *                    ^~~~~~'~$       it's guaranteed no overlap to rearrange the buffer,         *
                 *                   ^~~~~~~'$        and `buf_cap` can reset safely.                             *
                 *                          '                                                                     *
                 *                  ^~~~~~~~$         <- "worst" case.                                            *
                 *                         X'                                                                     *
                 *                          '                                                                     *
                 *  ^~~~~~~~$               '         After rearrangement.                                        *
                 *                                                                                                *
                 *  where the span is `off_consumed..off_read`.                                                   *
                 **************************************************************************************************/

                if *off_read - self.off_consumed >= Self::THRESHOLD {
                    return Ok(());
                }

                if *off_read >= Self::INIT_CAP - Self::THRESHOLD {
                    unsafe {
                        copy_nonoverlapping(
                            buf.as_ptr().add(self.off_consumed),
                            buf.as_ptr() as *mut _,
                            *off_read - self.off_consumed,
                        );
                    }

                    self.did_consumed += self.off_consumed;

                    *off_valid -= self.off_consumed;
                    *off_read -= self.off_consumed;

                    self.off_consumed = 0;

                    *buf_cap = Self::INIT_CAP;
                }
            }

            let len = rdr.read(unsafe { buf.get_unchecked_mut(*off_read..*buf_cap) })?;

            self.eof = len == 0;
            *off_read += len;

            return match self.eof {
                true => match *off_valid == *off_read {
                    true => Ok(()),
                    false => Err(Box::new(Utf8Error)),
                },
                false => match simdutf8::compat::from_utf8(unsafe { buf.get_unchecked(*off_valid..*off_read) }) {
                    Ok(_) => {
                        *off_valid = *off_read;
                        Ok(())
                    }
                    Err(e) => match e.error_len() {
                        None => continue,
                        Some(_) => Err(Box::new(Utf8Error)),
                    },
                },
            };
        }
    }

    /// Pull bytes, makes the [`content`](Self::content) has at least `n` bytes.
    ///
    /// Returns `Ok(false)` if encountered the EOF, unable to read such more bytes, or `n > 8192`.
    pub fn pull_at_least(&mut self, n: usize) -> TheResult<bool> {
        if n > Self::THRESHOLD {
            return Ok(false);
        }

        loop {
            let Source::Reader { off_valid, .. } = &self.src else {
                return Ok(self.content().len() >= n);
            };

            match *off_valid - self.off_consumed >= n {
                true => return Ok(true),
                false => match self.eof {
                    true => return Ok(false),
                    false => self.pull()?,
                },
            }
        }
    }

    //------------------------------------------------------------------------------

    #[inline]
    pub fn begin_select(&mut self) {
        self.selecting = true;
        self.off_selected = self.off_consumed;
        self.ctr_line_selected = self.ctr_line_consumed;
        self.ctr_column_selected = self.ctr_column_consumed;
    }

    #[inline]
    pub fn commit_select(&mut self) -> Option<&str> {
        self.selecting.then(|| {
            self.selecting = false;
            self.content_behind(self.off_selected..self.off_consumed)
        })
    }

    #[inline]
    pub fn rollback_select(&mut self) -> Option<&str> {
        self.selecting.then(|| {
            self.selecting = false;
            let span = self.off_selected..self.off_consumed;
            self.off_consumed = self.off_selected;
            self.ctr_line_consumed = self.ctr_line_selected;
            self.ctr_column_consumed = self.ctr_column_selected;
            self.content_behind(span)
        })
    }

    pub fn selection(&self) -> Option<&str> {
        self.selecting
            .then(|| self.content_behind(self.off_selected..self.off_consumed))
    }

    //------------------------------------------------------------------------------

    /// Consumes one character.
    ///
    /// This method will automatically [`pull`](Self::pull) only if the content is empty.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> TheResult<Option<char>> {
        if self.content().is_empty() {
            self.pull()?;
        }

        Ok(self.content().chars().next().inspect(|&ch| {
            if ch == '\n' {
                self.ctr_line_consumed = self.ctr_line_consumed.saturating_add(1);
                self.ctr_column_consumed = 0;
            } else {
                self.ctr_column_consumed += self.ctr_column_consumed.saturating_add(1);
            }

            self.off_consumed += ch.len_utf8();
        }))
    }

    /// Peeks one character.
    pub fn peek(&mut self) -> TheResult<Option<char>> {
        if self.content().is_empty() {
            self.pull()?;
        }

        Ok(self.content().chars().next())
    }

    /// Consumes one character then peeks the second if the previous call is still [`aheading`](Self::aheading),
    /// peeks one character otherwise.
    ///
    /// # Safety
    ///
    /// Requires manually let `self.lookahead = Peeked::None`,
    /// otherwise the offsets may not lie on UTF-8 sequence boundaries.
    ///
    /** Private method because opaque and unpinned internal offsets. */
    fn aheading(&mut self) -> TheResult<Option<char>> {
        if self.lookahead != Peeked::None {
            self.off_consumed += if self.lookahead == Peeked::Newline {
                self.ctr_line_consumed = self.ctr_line_consumed.saturating_add(1);
                self.ctr_column_consumed = 0;
                1
            } else {
                self.ctr_column_consumed = self.ctr_column_consumed.saturating_add(1);
                self.lookahead as usize
            };

            self.lookahead = Peeked::None;
        }

        if self.content().is_empty() {
            self.pull()?;
        }

        Ok(self
            .content()
            .chars()
            .next()
            .inspect(|&ch| self.lookahead = Peeked::from(ch)))
    }

    //------------------------------------------------------------------------------

    /// *(backtrack)* Consumes N characters.
    ///
    /// Returns `Ok(None)` and doesn't consume if encountered the EOF, unable to take such more characters.
    pub fn take(&mut self, n_char: usize) -> TheResult<Option<&str>> {
        self.locking = true;

        let start = self.off_consumed;
        for _ in 0..n_char {
            if self.next().inspect_err(|_| self.locking = false)?.is_none() {
                self.locking = false;

                self.off_consumed = start;

                return Ok(None);
            }
        }

        self.locking = false;

        Ok(Some(self.content_behind(start..self.off_consumed)))
    }

    /// *(backtrack)* Consumes one character if `predicate`.
    ///
    /// Returns `Ok(None)` if encountered the EOF.
    pub fn take_once<P>(&mut self, predicate: P) -> TheResult<Option<char>>
    where
        P: Predicate,
    {
        if let Some(ch) = self.peek()? {
            if ch == '\n' {
                self.ctr_line_consumed = self.ctr_line_consumed.saturating_add(1);
                self.ctr_column_consumed = 0;
            } else {
                self.ctr_column_consumed += self.ctr_column_consumed.saturating_add(1);
            }

            if predicate.predicate(ch) {
                self.off_consumed += ch.len_utf8();

                return Ok(Some(ch));
            }
        }

        Ok(None)
    }

    /// *(backtrack)* Consumes N..M characters consisting of `predicate`.
    ///
    /// Peeks the first unexpected character additionally, may be `None` if encountered the EOF.
    ///
    /// Returns `Ok(None)` and doesn't consume if the taking times not in `range`.
    pub fn take_times<P, U>(&mut self, predicate: P, range: U) -> TheResult<Option<(&str, Option<char>)>>
    where
        P: Predicate,
        U: URangeBounds,
    {
        self.locking = true;

        self.lookahead = Peeked::None;
        let mut times = 0;
        let start = self.off_consumed;
        let ch = loop {
            match self.aheading().inspect_err(|_| self.locking = false)? {
                None => break None,
                Some(ch) => match range.want_more(times) && predicate.predicate(ch) {
                    false => break Some(ch),
                    true => times += 1,
                },
            }
        };

        self.locking = false;

        match range.contains(times) {
            true => Ok(Some((self.content_behind(start..self.off_consumed), ch))),
            false => {
                self.off_consumed = start;
                Ok(None)
            }
        }
    }

    /// Consumes X characters consisting of `predicate`.
    ///
    /// Peeks the first unexpected character additionally, may be `None` if encountered the EOF.
    pub fn take_while<P>(&mut self, predicate: P) -> TheResult<(&str, Option<char>)>
    where
        P: Predicate,
    {
        self.locking = true;

        self.lookahead = Peeked::None;
        let start = self.off_consumed;
        let ch = loop {
            match self.aheading().inspect_err(|_| self.locking = false)? {
                None => break None,
                Some(ch) => match predicate.predicate(ch) {
                    false => break Some(ch),
                    true => continue,
                },
            }
        };

        self.locking = false;

        Ok((self.content_behind(start..self.off_consumed), ch))
    }

    /// *(backtrack)* Consumes K characters if matched `pattern`.
    ///
    /// Returns `Ok(None)` and doesn't consume if did't match anything.
    pub fn matches<P>(&mut self, pattern: P) -> TheResult<Option<P::Discriminant>>
    where
        P: Pattern,
    {
        self.pull_at_least(pattern.max_len())?;

        match pattern.matches(self.content()) {
            None => Ok(None),
            Some((len, discr)) => {
                self.off_consumed += len;
                Ok(Some(discr))
            }
        }
    }

    /// *(backtrack)* Consumes X characters until encountered `predicate`.
    ///
    /// The `predicate` is excluded from the returned slice, but is also consumed.
    ///
    /// Returns `Ok(None)` and doesn't consume if encountered the EOF.
    pub fn skim_till<P>(&mut self, predicate: P) -> TheResult<Option<(&str, char)>>
    where
        P: Predicate,
    {
        /* I'm bored to please the borrow checker */
        let off = addr_of_mut!(self.off_consumed);
        let (s, p) = self.skip_till(predicate)?;

        match p {
            Some(ch) => Ok(Some((s, ch))),
            None => unsafe {
                *off -= s.len();
                Ok(None)
            },
        }
    }

    /// *(backtrack)* Consumes X characters until encountered `pattern`.
    ///
    /// The `pattern` is excluded from the returned slice, but is also consumed.
    ///
    /// Returns `Ok(None)` and doesn't consume if encountered the EOF.
    pub fn skim_until<P>(&mut self, pattern: P) -> TheResult<Option<(&str, P::Discriminant)>>
    where
        P: Pattern,
    {
        /* I'm bored to please the borrow checker */
        let off = addr_of_mut!(self.off_consumed);
        let (s, p) = self.skip_until(pattern)?;

        match p {
            Some(discr) => Ok(Some((s, discr))),
            None => unsafe {
                *off -= s.len();
                Ok(None)
            },
        }
    }

    /// Consumes X characters until encountered `predicate`.
    ///
    /// Consumes the first encountered character additionally, may be `None` if encountered the EOF.
    pub fn skip_till<P>(&mut self, predicate: P) -> TheResult<(&str, Option<char>)>
    where
        P: Predicate,
    {
        self.locking = true;

        let start = self.off_consumed;
        let ch = loop {
            match self.next().inspect_err(|_| self.locking = false)? {
                None => break None,
                Some(ch) => match predicate.predicate(ch) {
                    true => break Some(ch),
                    false => continue,
                },
            }
        };

        self.locking = false;

        Ok((self.content_behind(start..self.off_consumed), ch))
    }

    /// Consumes X characters until encountered `pattern`.
    ///
    /// Consumes the first encountered sub-pattern additionally, may be `None` if encountered the EOF.
    pub fn skip_until<P>(&mut self, pattern: P) -> TheResult<(&str, Option<P::Discriminant>)>
    where
        P: Pattern,
    {
        self.locking = true;

        let start = self.off_consumed;
        let (span, discr) = loop {
            self.pull_at_least(pattern.max_len())
                .inspect_err(|_| self.locking = false)?;

            if self.exhausted() {
                let span = start..self.off_consumed;
                break (span, None);
            }

            if let Some((len, discr)) = pattern.matches(self.content()) {
                let span = start..self.off_consumed;
                self.off_consumed += len;
                break (span, Some(discr));
            }

            self.next()?;
        };

        self.locking = false;

        Ok((self.content_behind(span), discr))
    }
}
