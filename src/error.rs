use core::{fmt::Debug, num::NonZeroUsize};

pub trait Situation: Sized + Debug {
    type Description;

    fn unfulfilled(len: Option<NonZeroUsize>) -> Self;
    fn reject_at(len: usize) -> Self;
    fn halt_at(len: usize) -> Self;
    fn cut(self) -> Self;

    fn is_unfulfilled(&self) -> bool;
    fn is_rejected(&self) -> bool;
    fn is_halted(&self) -> bool;

    fn backtrack(self, len: usize) -> Self;
    fn length(&self) -> usize;

    fn describe(self, desc: Self::Description) -> Self;
    fn description(self) -> Self::Description;

    #[inline(always)]
    fn raise_unfulfilled<T>(len: Option<NonZeroUsize>) -> Result<T, Self> {
        Err(Self::unfulfilled(len))
    }
    #[inline(always)]
    fn raise_reject_at<T>(len: usize) -> Result<T, Self> {
        Err(Self::reject_at(len))
    }
    #[inline(always)]
    fn raise_halt_at<T>(len: usize) -> Result<T, Self> {
        Err(Self::halt_at(len))
    }

    #[inline(always)]
    fn raise_backtrack<T>(self, len: usize) -> Result<T, Self> {
        Err(self.backtrack(len))
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
    fn unfulfilled(len: Option<NonZeroUsize>) -> Self {
        Self {
            kind: SimpleErrorKind::Unfulfilled(len),
            desc: Default::default(),
        }
    }
    #[inline(always)]
    fn reject_at(len: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Rejected(len),
            desc: Default::default(),
        }
    }
    #[inline(always)]
    fn halt_at(len: usize) -> Self {
        Self {
            kind: SimpleErrorKind::Halted(len),
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
    fn backtrack(mut self, len: usize) -> Self {
        match &mut self.kind {
            SimpleErrorKind::Unfulfilled(_) => (),
            SimpleErrorKind::Rejected(n) => *n += len,
            SimpleErrorKind::Halted(n) => *n += len,
        }
        self
    }
    #[inline(always)]
    fn length(&self) -> usize {
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

//------------------------------------------------------------------------------

pub type ProviderResult<T, E = SimpleError> = Result<T, ProviderError<E>>;

#[derive(Debug)]
pub enum ProviderError<E: Situation> {
    #[cfg(feature = "std")]
    Io(::std::io::Error),
    InvalidUtf8,
    Mismatched(E),
}

#[cfg(feature = "std")]
impl<E: Situation> From<::std::io::Error> for ProviderError<E> {
    fn from(value: ::std::io::Error) -> Self {
        ProviderError::Io(value)
    }
}
