use crate::{
    common::{MaybeMut, MaybeRef},
    error::{EmptyErr, Error},
    input::*,
};
use core::{marker::PhantomData, ops::Range};

pub type State<S> = Full<EmptyErr, S, ()>;

pub type Context<C> = Full<EmptyErr, (), C>;

pub struct Full<E, S, C>(PhantomData<(E, S, C)>);

impl<'src, I, E, S, C> Extra<'src, I> for Full<E, S, C>
where
    I: Input<'src>,
    E: Error,
    S: 'src,
    C: 'src,
{
    type Error = E;
    type State = S;
    type Context = C;
}

pub trait Extra<'src, I>
where
    I: Input<'src>,
{
    type Error: Error;
    type State: 'src;
    type Context: 'src;
}

//------------------------------------------------------------------------------

pub struct ProvideExtra<'a, 'b, I, Ext>
where
    I: Input<'a>,
    Ext: Extra<'a, I>,
{
    input: &'b I,
    range: Range<I::Cursor>,
    state: MaybeMut<'b, Ext::State>,
    context: MaybeRef<'b, Ext::Context>,
}

impl<'a, 'b, I, Ext> ProvideExtra<'a, 'b, I, Ext>
where
    I: Input<'a>,
    Ext: Extra<'a, I>,
{
    pub fn span(&self) -> Range<usize> {
        self.input.span(self.range.clone())
    }

    pub fn offset(&self) -> usize {
        self.input.offset(self.range.start.clone())
    }

    pub fn slice(&self) -> &I::Slice
    where
        I: InputSlice<'a>,
    {
        self.input.get_slice(self.range.clone()).unwrap()
    }

    pub fn state(&mut self) -> &mut Ext::State {
        &mut self.state
    }

    pub fn context(&self) -> &Ext::Context {
        &self.context
    }
}
