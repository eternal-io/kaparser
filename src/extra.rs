use crate::{error::Error, input::Input};

pub trait Extra<'src, I>
where
    I: Input<'src>,
{
    // type Error: Error + From<I::Error> + 'src;
    type State: 'src;
    type Context: 'src;
}
