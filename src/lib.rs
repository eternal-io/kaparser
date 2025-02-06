#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
use core::{
    error::Error,
    fmt, mem,
    num::{NonZeroU16, NonZeroU8, NonZeroUsize},
    ops::Range,
    ptr,
    str::from_utf8_unchecked,
};
use simdutf8::compat::from_utf8;

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

pub struct Utf8Parser<'source, R: Read> {
    src: Source<'source, R>,
    eof: bool,

    /* It's decided not to provide nested-select functionality,
     * because kaparser just works at the lowest level. */
    off_selected: Option<NonZeroUsize>,
    off_consumed: usize,
    did_consumed: usize,

    ctr_line: u16,
    ctr_line_select: u16,
    /* column count in characters, not bytes */
    ctr_column: u16,
    ctr_column_select: u16,

    peeked: Option<NonZeroU8>,
}

#[derive(Debug)]
pub struct Utf8Error {
    pub(crate) position: usize,
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

//==================================================================================================

impl Read for Slice {
    fn read(&mut self, _buf: &mut [u8]) -> TheResult<usize> {
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

impl Utf8Error {
    pub fn position(&self) -> usize {
        self.position
    }
}

impl Error for Utf8Error {}

impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("invalid UTF-8 bytes at {}", self.position))
    }
}

impl<'source> Utf8Parser<'source, Slice> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(slice: &'source str) -> Self {
        Self {
            src: Source::Borrowed { slice },
            eof: true,

            off_selected: None,
            off_consumed: 0,
            did_consumed: 0,

            ctr_line: 0,
            ctr_line_select: 0,
            ctr_column: 0,
            ctr_column_select: 0,

            peeked: None,
        }
    }

    pub fn from_bytes(bytes: &'source [u8]) -> Result<Self, Utf8Error> {
        from_utf8(bytes).map(Self::from_str).map_err(|e| Utf8Error {
            position: e.valid_up_to(),
        })
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

            off_selected: None,
            off_consumed: 0,
            did_consumed: 0,

            ctr_line: 0,
            ctr_line_select: 0,
            ctr_column: 0,
            ctr_column_select: 0,

            peeked: None,
        }
    }
}

impl<R: Read> Utf8Parser<'_, R> {
    const INIT_CAP: usize = 32 * 1024;
    const THRES_REARRANGE: usize = 8 * 1024;

    /// TODO! Marks the leading `n` bytes of the content as consumed, they will disappear in the future content.
    ///
    /// # Panics
    ///
    /// Panics if the `n`th byte is not at a UTF-8 character boundary.
    pub fn bump(&mut self, n: usize) {
        if !self.content().is_char_boundary(n) {
            panic!("{} is not at a UTF-8 character boundary", n)
        }

        self.__bump(n);
    }

    #[inline]
    fn __bump(&mut self, n: usize) {
        if self
            .off_selected
            .as_mut()
            .map(|off| *off = off.saturating_add(n))
            .is_none()
        {
            self.off_consumed += n;
        }
    }

    #[inline]
    fn __content(&self, span: Range<usize>) -> &str {
        unsafe {
            match &self.src {
                Source::Borrowed { slice } => slice.get_unchecked(span),
                Source::Reader { buf, .. } => from_utf8_unchecked(buf.get_unchecked(span)),
            }
        }
    }

    /// Returns the unconsumed content.
    #[inline]
    pub fn content(&self) -> &str {
        let start = self.off_selected.map(usize::from).unwrap_or(self.off_consumed);
        unsafe {
            match &self.src {
                Source::Borrowed { slice } => slice.get_unchecked(start..),
                Source::Reader { buf, off_valid, .. } => from_utf8_unchecked(buf.get_unchecked(start..*off_valid)),
            }
        }
    }

    /// Returns all the buffered content, includes those consumed,
    /// with the start offset of this slice among the overall input.
    pub fn buffered_content(&self) -> (usize, &str) {
        (
            self.did_consumed,
            match &self.src {
                Source::Borrowed { slice } => slice,
                Source::Reader { buf, off_valid, .. } => unsafe {
                    from_utf8_unchecked(buf.get_unchecked(..*off_valid))
                },
            },
        )
    }

    /// TODO! Returns the count of totally consumed bytes (excludes selection).
    pub fn consumed(&self) -> usize {
        self.did_consumed + self.off_consumed
    }

    /// TODO! Returns `true` if all bytes are consumed (includes selection) and encountered the EOF.
    pub fn exhausted(&self) -> bool {
        let start = self.off_selected.map(usize::from).unwrap_or(self.off_consumed);
        match &self.src {
            Source::Borrowed { slice } => start == slice.len(),
            Source::Reader { off_valid, .. } => self.eof && start == *off_valid,
        }
    }

    //------------------------------------------------------------------------------

    pub fn raise_error<T, E>(&self, mut e: E) -> Result<T, E>
    where
        E: Situate,
    {
        let to = (
            NonZeroU16::try_from(self.ctr_line.saturating_add(1)).unwrap(),
            NonZeroU16::try_from(self.ctr_column.saturating_add(1)).unwrap(),
        );
        let from = self.off_selected.is_some().then(|| {
            (
                NonZeroU16::try_from(self.ctr_line_select.saturating_add(1)).unwrap(),
                NonZeroU16::try_from(self.ctr_column_select.saturating_add(1)).unwrap(),
            )
        });

        e.situate(to, from);

        Err(e)
    }

    //------------------------------------------------------------------------------

    pub fn pull_smart(&mut self) -> TheResult<()> {
        todo!()
    }

    /// Pull bytes if available content is less than 8 KiB.
    ///
    /*  WARN: The offset of content may NOT be pinned. */
    pub fn pull(&mut self) -> TheResult<()> {
        let Source::Reader {
            buf,
            buf_cap,
            off_read,
            off_valid,
            ..
        } = &mut self.src
        else {
            return Ok(());
        };

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

        if *off_read - self.off_consumed > Self::THRES_REARRANGE {
            return Ok(());
        }

        if *off_read >= Self::INIT_CAP - Self::THRES_REARRANGE {
            unsafe {
                ptr::copy_nonoverlapping(
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

        self.fetch(Self::pull)
    }

    /// Pull more bytes, allows the content to grow infinitely.
    ///
    /*  NOTE: The offset of content would be pinned. */
    pub fn pull_more(&mut self) -> TheResult<()> {
        let Source::Reader {
            buf, buf_cap, off_read, ..
        } = &mut self.src
        else {
            return Ok(());
        };

        fn m7d8(n: usize) -> usize {
            (n >> 1) + (n >> 2) + (n >> 3)
        }

        if *off_read > m7d8(*buf_cap) {
            *buf_cap <<= 1;
            if *buf_cap > buf.len() {
                let mut buf_new = unsafe { Box::new_uninit_slice(*buf_cap).assume_init() };
                unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), buf_new.as_mut_ptr(), *off_read) }
                drop(mem::replace(buf, buf_new));
            }
        }

        self.fetch(Self::pull_more)
    }

    /// Pull more bytes, makes the content has at least `n` bytes.
    ///
    /// Returns `Ok(false)` if encountered the EOF, unable to read such more bytes.
    ///
    /*  NOTE: The offset of content would be pinned.  */
    pub fn pull_at_least(&mut self, n: usize) -> TheResult<bool> {
        loop {
            let Source::Reader { off_valid, .. } = &self.src else {
                return Ok(self.content().len() >= n);
            };

            match self.off_consumed + n > *off_valid {
                false => return Ok(true),
                true => match !self.eof {
                    false => return Ok(false),
                    true => self.pull_more()?,
                },
            }
        }
    }

    fn fetch(&mut self, rerun: fn(&mut Self) -> TheResult<()>) -> TheResult<()> {
        let Source::Reader {
            rdr,
            buf,
            buf_cap,
            off_read,
            off_valid,
        } = &mut self.src
        else {
            unreachable!()
        };

        let len = unsafe { rdr.read(buf.get_unchecked_mut(*off_read..*buf_cap))? };

        self.eof = len == 0;

        if !self.eof {
            *off_read += len;
            match self.validate()? {
                true => Ok(()),
                false => rerun(self),
            }
        } else {
            match *off_valid == *off_read {
                true => Ok(()),
                false => Err(Box::new(Utf8Error {
                    position: self.did_consumed + *off_valid + 1,
                })),
            }
        }
    }

    fn validate(&mut self) -> TheResult<bool> {
        let Source::Reader {
            buf,
            off_read,
            off_valid,
            ..
        } = &mut self.src
        else {
            unreachable!()
        };

        if let Err(e) = unsafe { from_utf8(buf.get_unchecked(*off_valid..*off_read)) } {
            match e.error_len() {
                None => Ok(false),
                Some(_) => Err(Box::new(Utf8Error {
                    position: self.did_consumed + *off_valid + e.valid_up_to() + 1,
                })),
            }
        } else {
            *off_valid = *off_read;
            Ok(true)
        }
    }

    //------------------------------------------------------------------------------

    pub fn select_begin(&mut self) {
        // self.off_selected.insert(self.off_consumed);
        todo!()
    }

    pub fn select_commit(&mut self) -> Option<&str> {
        let start = self.off_consumed;
        self.off_selected.take().map(usize::from).map(|off| {
            self.off_consumed = off;
            self.__content(start..off)
        })
    }

    pub fn select_rollback(&mut self) -> Option<&str> {
        self.off_selected
            .take()
            .map(usize::from)
            .map(|off| self.__content(self.off_consumed..off))
    }

    pub fn selection(&self) -> Option<&str> {
        self.off_selected
            .map(usize::from)
            .map(|off| self.__content(self.off_consumed..off))
    }

    //------------------------------------------------------------------------------

    /// Consume one character.
    ///
    /// This method will automatically [`pull`](Self::pull) if the content is empty.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> TheResult<Option<char>> {
        self.pull()?;

        Ok(self.content().chars().next().inspect(|ch| {
            self.off_consumed += ch.len_utf8();
        }))
    }

    /// Peeks one character.
    ///
    /// This method will automatically [`pull`](Self::pull) if the content is empty.
    pub fn peek(&mut self) -> TheResult<Option<char>> {
        self.pull()?;

        Ok(self.content().chars().next())
    }

    /// As same as the [`next`](Self::next), but [`pull_more`](Self::pull_more) instead.
    ///
    /** Private method because opaque and unpinned internal offsets. */
    #[inline(always)]
    fn nexting(&mut self) -> TheResult<Option<char>> {
        if self.content().is_empty() {
            self.pull_more()?;
        }

        Ok(self.content().chars().next().inspect(|ch| {
            self.off_consumed += ch.len_utf8();
        }))
    }

    /// Consume one character then peeks the second if the previous call is still [`peeking`](Self::peeking),
    /// peeks one character otherwise.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    ///
    /// NOTE: Needs manually let `self.peeked = None`.
    ///
    /** Private method because opaque and unpinned internal offsets. */
    #[inline(always)]
    fn peeking(&mut self) -> TheResult<Option<char>> {
        if let Some(len) = self.peeked.take() {
            self.off_consumed += u8::from(len) as usize;
        }

        if self.content().is_empty() {
            self.pull_more()?;
        }

        Ok(self.content().chars().next().inspect(|ch| {
            self.peeked = Some((ch.len_utf8() as u8).try_into().unwrap());
        }))
    }

    //------------------------------------------------------------------------------

    /// Consume N characters.
    ///
    /// Returns `Ok(Err(_))` if encountered the EOF.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    pub fn take(&mut self, n_char: usize) -> TheResult<Result<&str, &str>> {
        let start = self.off_consumed;

        for _ in 0..n_char {
            if self.nexting()?.is_none() {
                return Ok(Err(self.__content(start..self.off_consumed)));
            }
        }

        Ok(Ok(self.__content(start..self.off_consumed)))
    }

    /// Consume one character if `predicate`.
    ///
    /// Returns `Ok(None)` if encountered the EOF.
    ///
    /// This method will automatically [`pull`](Self::pull) if the content is empty.
    pub fn take_once<P>(&mut self, predicate: P) -> TheResult<Option<char>>
    where
        P: Predicate,
    {
        Ok(match self.peek()? {
            None => None,
            Some(ch) => match predicate.predicate(ch) {
                false => None,
                true => {
                    self.off_consumed += ch.len_utf8();
                    Some(ch)
                }
            },
        })
    }

    /// Consume N..M characters consisting of `predicate`.
    ///
    /// Peeks the first unexpected character additionally, may be `None` if encountered the EOF.
    ///
    /// Returns `Ok(Err(_))` and doesn't consume if the taking times not in `range`.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    #[allow(clippy::type_complexity)]
    pub fn take_times<P, U>(
        &mut self,
        predicate: P,
        range: U,
    ) -> TheResult<Result<(&str, Option<char>), (&str, Option<char>)>>
    where
        P: Predicate,
        U: URangeBounds,
    {
        self.peeked = None;

        let mut times = 0;
        let start = self.off_consumed;
        let ch = loop {
            match self.peeking()? {
                None => break None,
                Some(ch) => match range.want_more(times) && predicate.predicate(ch) {
                    false => break Some(ch),
                    true => times += 1,
                },
            }
        };

        let span = start..self.off_consumed;

        Ok(match range.contains(times) {
            true => Ok((self.__content(span), ch)),
            false => {
                self.off_consumed = start;
                Err((self.__content(span), ch))
            }
        })
    }

    /// Consume X characters consisting of `predicate`.
    ///
    /// Peeks the first unexpected character additionally, may be `None` if encountered the EOF.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    pub fn take_while<P>(&mut self, predicate: P) -> TheResult<(&str, Option<char>)>
    where
        P: Predicate,
    {
        self.peeked = None;

        let start = self.off_consumed;
        let ch = loop {
            match self.peeking()? {
                None => break None,
                Some(ch) => match predicate.predicate(ch) {
                    false => break Some(ch),
                    true => continue,
                },
            }
        };

        Ok((self.__content(start..self.off_consumed), ch))
    }

    /// Consume K characters if matched `pattern`.
    ///
    /// Returns `Ok(None)` and doesn't consume if did't match anything.
    ///
    /// This method will automatically [`pull`](Self::pull) if the content is insufficient.
    pub fn matches<P>(&mut self, pattern: P) -> TheResult<Option<P::Discriminant>>
    where
        P: Pattern,
    {
        self.pull_at_least(pattern.max_len())?;

        Ok(match pattern.matches(self.content()) {
            None => None,
            Some((len, discr)) => {
                self.off_consumed += len;

                Some(discr)
            }
        })
    }

    /// Consume X characters until encountered `predicate`.
    ///
    /// The `predicate` is excluded from the result and also marked as consumed.
    ///
    /// Returns `Ok(Err(_))` and doesn't consume if encountered the EOF.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    pub fn skim_till<P>(&mut self, predicate: P) -> TheResult<Result<(&str, char), &str>>
    where
        P: Predicate,
    {
        self.peeked = None;

        let start = self.off_consumed;
        let ch = loop {
            match self.peeking()? {
                None => break None,
                Some(ch) => match !predicate.predicate(ch) {
                    false => break Some(ch),
                    true => continue,
                },
            }
        };

        let spanned = self.__content(start..self.off_consumed);

        Ok(match ch {
            Some(ch) => Ok((spanned, ch)),
            None => Err(spanned),
        })
    }

    /// Consume X characters until encountered `pattern`.
    ///
    /// The `pattern` is excluded from the result and also marked as consumed.
    ///
    /// Returns `Ok(Err(_))` and doesn't consume if encountered the EOF.
    ///
    /// This method will automatically [`pull_more`](Self::pull_more) if the content is insufficient.
    pub fn skim_until<P>(&mut self, pattern: P) -> TheResult<Result<(&str, P::Discriminant), &str>>
    where
        P: Pattern,
    {
        let start = self.off_consumed;

        loop {
            self.pull_at_least(pattern.max_len())?;

            if self.exhausted() {
                let span = start..self.off_consumed;

                self.off_consumed = start;

                return Ok(Err(self.__content(span)));
            }

            match pattern.matches(self.content()) {
                Some((len, discr)) => {
                    let span = start..self.off_consumed;

                    self.off_consumed += len;

                    return Ok(Ok((self.__content(span), discr)));
                }
                None => {
                    self.next()?;
                }
            }
        }
    }

    /// Deprecate X characters until encountered `predicate`.
    ///
    /// Peeks the first encountered character additionally, may be `None` if encountered the EOF.
    pub fn skip_till<P>(&mut self, predicate: P) -> TheResult<Option<char>>
    where
        P: Predicate,
    {
        self.peeked = None;

        Ok(loop {
            match self.peeking()? {
                None => break None,
                Some(ch) => match !predicate.predicate(ch) {
                    false => break Some(ch),
                    true => continue,
                },
            }
        })
    }

    /// Deprecate X characters until encountered `pattern`.
    ///
    /// Peeks the first encountered sub-pattern additionally, may be `None` if encountered the EOF.
    pub fn skip_until<P>(&mut self, pattern: P) -> TheResult<Option<P::Discriminant>>
    where
        P: Pattern,
    {
        loop {
            self.pull_at_least(pattern.max_len())?;

            if self.exhausted() {
                return Ok(None);
            }

            match pattern.matches(self.content()) {
                Some((len, discr)) => {
                    self.off_consumed += len;

                    return Ok(Some(discr));
                }
                None => {
                    self.next()?;
                }
            }
        }
    }
}
