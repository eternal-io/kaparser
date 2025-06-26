use crate::{extra::Extra, input::Input};
use core::{
    fmt::{self, Debug, Display},
    ops::Range,
};

pub trait Error: Sized + Debug {
    fn new(span: Range<usize>, kind: ErrorKind) -> Self;

    fn merge(self, newer: Self) -> Self {
        #![allow(unused_variables)]
        self
    }
}

//------------------------------------------------------------------------------

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    #[doc(hidden)]
    #[cfg(feature = "alloc")]
    __Display(alloc::boxed::Box<dyn core::error::Error>),

    #[doc(hidden)]
    #[cfg(not(feature = "alloc"))]
    __Display,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

//------------------------------------------------------------------------------

impl<'src, I, E> Extra<'src, I> for E
where
    I: Input<'src>,
    E: Error,
{
    type Error = E;
    type State = ();
    type Context = ();
}

//------------------------------------------------------------------------------

pub struct EmptyErr;
