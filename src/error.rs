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
    fn description(&self) -> Self::Description;

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

#[derive(Debug, Clone, Copy)]
pub enum SimpleError {
    Unfulfilled(Option<NonZeroUsize>),
    Rejected(usize),
    Halted(usize),
}

impl Situation for SimpleError {
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
    fn cut(self) -> Self {
        match self {
            SimpleError::Rejected(n) => SimpleError::Halted(n),
            _ => self,
        }
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
    fn backtrack(mut self, len: usize) -> Self {
        match &mut self {
            Self::Unfulfilled(_) => (),
            Self::Rejected(n) => *n += len,
            Self::Halted(n) => *n += len,
        }
        self
    }
    #[inline(always)]
    fn length(&self) -> usize {
        match *self {
            Self::Unfulfilled(n) => n.map(usize::from).unwrap_or(0),
            Self::Rejected(n) => n,
            Self::Halted(n) => n,
        }
    }

    #[inline(always)]
    fn describe(self, _esc: Self::Description) -> Self {
        self
    }
    #[inline(always)]
    fn description(&self) -> Self::Description {}
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
