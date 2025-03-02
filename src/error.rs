use core::{fmt::Debug, num::NonZeroUsize};

pub trait Situation: Sized + Debug {
    fn unfulfilled(delta: Option<NonZeroUsize>) -> Self;
    fn reject_at(delta: usize) -> Self;
    fn halt_at(delta: usize) -> Self;

    fn is_unfulfilled(&self) -> bool;
    fn is_rejected(&self) -> bool;
    fn is_halted(&self) -> bool;

    fn delta(&self) -> usize;

    #[inline(always)]
    fn raise_unfulfilled<T>(delta: Option<NonZeroUsize>) -> Result<T, Self> {
        Err(Self::unfulfilled(delta))
    }
    #[inline(always)]
    fn raise_reject_at<T>(delta: usize) -> Result<T, Self> {
        Err(Self::reject_at(delta))
    }
    #[inline(always)]
    fn raise_halt_at<T>(delta: usize) -> Result<T, Self> {
        Err(Self::halt_at(delta))
    }
}

pub trait DetailedSituation: Situation {
    fn msg(self, msg: &str) -> Self;
    fn expected(self, expected: &str) -> Self;
}

//------------------------------------------------------------------------------

pub type PrecedeResult<E = SimpleError> = Result<usize, E>;

//------------------------------------------------------------------------------

pub type SimpleResult<T, E = SimpleError> = Result<T, E>;

#[derive(Debug, Clone, Copy)]
pub enum SimpleError {
    Unfulfilled(Option<NonZeroUsize>),
    Rejected(usize),
    Halted(usize),
}

impl Situation for SimpleError {
    #[inline(always)]
    fn unfulfilled(delta: Option<NonZeroUsize>) -> Self {
        Self::Unfulfilled(delta)
    }
    #[inline(always)]
    fn reject_at(delta: usize) -> Self {
        Self::Rejected(delta)
    }
    #[inline(always)]
    fn halt_at(delta: usize) -> Self {
        Self::Halted(delta)
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
    fn delta(&self) -> usize {
        match *self {
            Self::Unfulfilled(n) => n.map(usize::from).unwrap_or(0),
            Self::Rejected(n) => n,
            Self::Halted(n) => n,
        }
    }
}

//------------------------------------------------------------------------------

pub type ParseResult<T, E = SimpleError> = Result<T, ParseError<E>>;

#[derive(Debug)]
pub enum ParseError<E: Situation> {
    #[cfg(feature = "std")]
    Io(::std::io::Error),
    InvalidUtf8,
    Mismatched(E),
}

#[cfg(feature = "std")]
impl<E: Situation> From<::std::io::Error> for ParseError<E> {
    fn from(value: ::std::io::Error) -> Self {
        ParseError::Io(value)
    }
}
