use core::{fmt::Debug, num::NonZeroUsize};

pub trait Situation: Sized + Debug {
    type Description;

    fn unfulfilled(len: Option<NonZeroUsize>) -> Self;
    fn reject_at(len: usize) -> Self;
    fn halt_at(len: usize) -> Self;

    fn backtrack(self, len: usize) -> Self;
    fn detail(self, desc: Self::Description) -> Self;

    fn is_unfulfilled(&self) -> bool;
    fn is_rejected(&self) -> bool;
    fn is_halted(&self) -> bool;
    fn length(&self) -> usize;

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
}

//------------------------------------------------------------------------------

pub type PrecedeResult<E = ParseError> = Result<usize, E>;

//------------------------------------------------------------------------------

pub type ParseResult<T, E = ParseError> = Result<T, E>;

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    Unfulfilled(Option<NonZeroUsize>),
    Rejected(usize),
    Halted(usize),
}

impl Situation for ParseError {
    type Description = ();

    #[inline(always)]
    fn unfulfilled(len: Option<NonZeroUsize>) -> Self {
        Self::Unfulfilled(len)
    }
    #[inline(always)]
    fn reject_at(len: usize) -> Self {
        Self::Rejected(len)
    }
    #[inline(always)]
    fn halt_at(len: usize) -> Self {
        Self::Halted(len)
    }

    #[inline(always)]
    fn backtrack(mut self, len: usize) -> Self {
        match &mut self {
            Self::Unfulfilled(_) => (),
            Self::Rejected(n) => *n += len,
            Self::Halted(n) => *n += len,
        }
        self
    }
    #[inline(always)]
    fn detail(self, _desc: Self::Description) -> Self {
        self
    }

    #[inline(always)]
    fn is_unfulfilled(&self) -> bool {
        matches!(self, Self::Unfulfilled(_))
    }
    #[inline(always)]
    fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected(_))
    }
    #[inline(always)]
    fn is_halted(&self) -> bool {
        matches!(self, Self::Halted(_))
    }
    #[inline(always)]
    fn length(&self) -> usize {
        match *self {
            Self::Unfulfilled(n) => n.map(usize::from).unwrap_or(0),
            Self::Rejected(n) => n,
            Self::Halted(n) => n,
        }
    }
}

//------------------------------------------------------------------------------

pub type ProvideResult<T, E = ParseError> = Result<T, ProvideError<E>>;

#[derive(Debug)]
pub enum ProvideError<E: Situation> {
    #[cfg(feature = "std")]
    Io(::std::io::Error),
    InvalidUtf8,
    Mismatched(E),
}

#[cfg(feature = "std")]
impl<E: Situation> From<::std::io::Error> for ProvideError<E> {
    fn from(value: ::std::io::Error) -> Self {
        ProvideError::Io(value)
    }
}
