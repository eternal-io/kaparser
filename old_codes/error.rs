use core::{fmt::Debug, num::NonZeroUsize};

pub trait Situation: Sized + Debug {
    type Description;

    /// Indicates the current pattern requires longer input. This only occurs if the `eof` flag is `false`.
    ///
    /// A `None` value indicates that it is not possible to determine how long the "longer" is, but just requires longer input.
    fn unfulfilled(ext: Option<NonZeroUsize>) -> Self;
    /// Indicates the current pattern does not match the input.
    fn reject_at(off: usize) -> Self;
    /// Indicates the current pattern does not match the input, and no other alternative branches should be tried.
    fn halt_at(off: usize) -> Self;
    /// Turn `REJECT` to `HALT`.
    fn cut(self) -> Self;

    fn is_unfulfilled(&self) -> bool;
    fn is_rejected(&self) -> bool;
    fn is_halted(&self) -> bool;

    /// Record the offset of the current situation on the current input.
    ///
    /// This is done for all nested patterns, so the correct offset of the situation relative to the original input is known.
    fn backtrack(self, off: usize) -> Self;
    /// Get the offset of the situation relative to the starting position of the input.
    fn offset(&self) -> usize;

    /// Set the description for the situation.
    fn describe(self, desc: Self::Description) -> Self;
    /// Get the description of the situation.
    fn description(self) -> Self::Description;

    #[inline]
    fn raise_unfulfilled<T>(ext: Option<NonZeroUsize>) -> Result<T, Self> {
        Err(Self::unfulfilled(ext))
    }
    #[inline]
    fn raise_reject_at<T>(off: usize) -> Result<T, Self> {
        Err(Self::reject_at(off))
    }
    #[inline]
    fn raise_halt_at<T>(off: usize) -> Result<T, Self> {
        Err(Self::halt_at(off))
    }

    #[inline]
    fn raise_backtrack<T>(self, off: usize) -> Result<T, Self> {
        Err(self.backtrack(off))
    }
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct SimpleError<D = ()>
where
    D: Default + Debug,
{
    kind: SimpleErrorKind,
    desc: D,
}

#[derive(Debug, Clone, Copy)]
enum SimpleErrorKind {
    Unfulfilled(Option<NonZeroUsize>),
    Rejected(usize),
    Halted(usize),
}

impl<D> Situation for SimpleError<D>
where
    D: Default + Debug,
{
    type Description = D;

    #[inline]
    fn unfulfilled(ext: Option<NonZeroUsize>) -> Self {
        Self {
            kind: SimpleErrorKind::Unfulfilled(ext),
            desc: Default::default(),
        }
    }
    #[inline]
    fn reject_at(off: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Rejected(off),
            desc: Default::default(),
        }
    }
    #[inline]
    fn halt_at(off: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Halted(off),
            desc: Default::default(),
        }
    }

    #[inline]
    fn cut(self) -> Self {
        let Self { kind, desc } = self;
        match kind {
            SimpleErrorKind::Rejected(n) => Self {
                kind: SimpleErrorKind::Halted(n),
                desc,
            },
            _ => Self { kind, desc },
        }
    }

    #[inline]
    fn is_unfulfilled(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Unfulfilled(_))
    }
    #[inline]
    fn is_rejected(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Rejected(_))
    }
    #[inline]
    fn is_halted(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Halted(_))
    }

    #[inline]
    fn backtrack(mut self, off: usize) -> Self {
        match &mut self.kind {
            SimpleErrorKind::Unfulfilled(_) => (),
            SimpleErrorKind::Rejected(n) => *n += off,
            SimpleErrorKind::Halted(n) => *n += off,
        }
        self
    }
    #[inline]
    fn offset(&self) -> usize {
        match self.kind {
            SimpleErrorKind::Unfulfilled(n) => n.map(usize::from).unwrap_or(0),
            SimpleErrorKind::Rejected(n) => n,
            SimpleErrorKind::Halted(n) => n,
        }
    }

    #[inline]
    fn describe(mut self, desc: Self::Description) -> Self {
        self.desc = desc;
        self
    }
    #[inline]
    fn description(self) -> Self::Description {
        self.desc
    }
}
