use core::num::NonZeroUsize;

pub type PrecedeResult<E = SimpleError> = Result<usize, E>;

//------------------------------------------------------------------------------

pub type SimpleResult<T, E = SimpleError> = Result<T, E>;

#[derive(Debug)]
pub enum SimpleError {}

pub trait Situation: Sized {
    fn unfulfilled(delta: Option<NonZeroUsize>) -> Self;
    fn reject_at(delta: usize) -> Self;
    fn halt_at(delta: usize) -> Self;

    fn is_unfulfilled(&self) -> bool;
    fn is_rejected(&self) -> bool;
    fn is_halted(&self) -> bool;

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
