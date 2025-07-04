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

pub struct ProvideExtra<'src, 'tmp, I, Ext>
where
    'src: 'tmp,
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    input: &'tmp I,
    range: Range<I::Cursor>,
    state: MaybeMut<'tmp, Ext::State>,
    context: MaybeRef<'tmp, Ext::Context>,
}

impl<'src, 'tmp, I, Ext> ProvideExtra<'src, 'tmp, I, Ext>
where
    'src: 'tmp,
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    pub fn span(&self) -> Range<usize> {
        I::span(self.range.clone())
    }
    pub fn offset(&self) -> usize {
        I::offset(self.range.start.clone())
    }

    pub fn slice(&self) -> &'tmp I::Slice
    where
        I: InputSlice<'src>,
    {
        self.input.get_slice(self.range.clone()).unwrap()
    }
    pub fn slice_outlived(&self) -> &'src I::Slice
    where
        I: InputSlice<'src> + StaticInput,
    {
        // SAFETY: TODO! See StaticInput.
        unsafe { core::mem::transmute::<&'tmp I::Slice, &'src I::Slice>(self.slice()) }
    }

    pub fn state(&mut self) -> &mut Ext::State {
        &mut self.state
    }
    pub fn context(&self) -> &Ext::Context {
        &self.context
    }
}
