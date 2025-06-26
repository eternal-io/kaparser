use crate::{error::Error, input::Input};

pub trait Extra<'src, I>
where
    I: Input<'src>,
{
    type Error: Error;
    type State: 'src;
    type Context: 'src;
}
