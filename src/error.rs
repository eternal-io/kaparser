use core::fmt::Debug;

use crate::{extra::Extra, input::Input};

pub trait Error: Debug {}

// impl<'src, I, E> Extra<'src, I> for E
// where
//     I: Input<'src>,
//     E: Error + From<I::Error> + 'src,
// {
//     type Error = E;
//     type State = ();
//     type Context = ();
// }
