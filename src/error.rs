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

    #[inline(always)]
    fn raise_unfulfilled<T>(ext: Option<NonZeroUsize>) -> Result<T, Self> {
        Err(Self::unfulfilled(ext))
    }
    #[inline(always)]
    fn raise_reject_at<T>(off: usize) -> Result<T, Self> {
        Err(Self::reject_at(off))
    }
    #[inline(always)]
    fn raise_halt_at<T>(off: usize) -> Result<T, Self> {
        Err(Self::halt_at(off))
    }

    #[inline(always)]
    fn raise_backtrack<T>(self, off: usize) -> Result<T, Self> {
        Err(self.backtrack(off))
    }
}

//------------------------------------------------------------------------------

pub type ParseResult<T, E = SimpleError> = Result<T, E>;

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

    #[inline(always)]
    fn unfulfilled(ext: Option<NonZeroUsize>) -> Self {
        Self {
            kind: SimpleErrorKind::Unfulfilled(ext),
            desc: Default::default(),
        }
    }
    #[inline(always)]
    fn reject_at(off: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Rejected(off),
            desc: Default::default(),
        }
    }
    #[inline(always)]
    fn halt_at(off: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Halted(off),
            desc: Default::default(),
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    fn is_unfulfilled(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Unfulfilled(_))
    }
    #[inline(always)]
    fn is_rejected(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Rejected(_))
    }
    #[inline(always)]
    fn is_halted(&self) -> bool {
        matches!(self.kind, SimpleErrorKind::Halted(_))
    }

    #[inline(always)]
    fn backtrack(mut self, off: usize) -> Self {
        match &mut self.kind {
            SimpleErrorKind::Unfulfilled(_) => (),
            SimpleErrorKind::Rejected(n) => *n += off,
            SimpleErrorKind::Halted(n) => *n += off,
        }
        self
    }
    #[inline(always)]
    fn offset(&self) -> usize {
        match self.kind {
            SimpleErrorKind::Unfulfilled(n) => n.map(usize::from).unwrap_or(0),
            SimpleErrorKind::Rejected(n) => n,
            SimpleErrorKind::Halted(n) => n,
        }
    }

    #[inline(always)]
    fn describe(mut self, desc: Self::Description) -> Self {
        self.desc = desc;
        self
    }
    #[inline(always)]
    fn description(self) -> Self::Description {
        self.desc
    }
}
