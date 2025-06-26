use crate::{extra::Extra, input::Input};
use core::fmt::{Debug, Display};

pub trait Error: Sized + Debug {
    fn push_runtime<T: Display>(self, msg: T) -> Self;
}

impl<'src, I, E> Extra<'src, I> for E
where
    I: Input<'src>,
    E: Error + 'src,
{
    type Error = E;
    type State = ();
    type Context = ();
}

//------------------------------------------------------------------------------

pub struct EmptyErr;
