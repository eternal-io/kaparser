use crate::common::Describe;
use core::{
    fmt::{self, Debug},
    ops::Range,
};

pub trait Error: Sized + Debug {
    type Label;

    fn new(span: Range<usize>, kind: ErrorKind) -> Self;

    fn merge(self, other: Self) -> Self;

    fn label(self, label: Self::Label) -> Self;
}

//------------------------------------------------------------------------------

#[non_exhaustive]
pub enum ErrorKind<'a> {
    Expected(&'a dyn Describe),
    ExpectedEnd,
    Other(&'a dyn core::error::Error),

    // non-fatal kinds.
    InvalidInput,
}

impl<'a> Describe for ErrorKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Expected(pat) => write!(f, "expected {:?}", pat),
            ErrorKind::ExpectedEnd => write!(f, "expected end of input"),
            ErrorKind::Other(err) => write!(f, "error: {}", err),

            ErrorKind::InvalidInput => write!(f, "invalid input"),
        }
    }
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct EmptyErr;

impl Error for EmptyErr {
    type Label = ();

    fn new(span: Range<usize>, kind: ErrorKind) -> Self {
        #![allow(unused_variables)]
        Self
    }

    fn merge(self, other: Self) -> Self {
        #![allow(unused_variables)]
        Self
    }

    fn label(self, label: Self::Label) -> Self {
        #![allow(unused_variables)]
        Self
    }
}
