use crate::{
    error::{EmptyErr, Error},
    input::Input,
};
use core::marker::PhantomData;

pub(crate) type ProvideExtra<'src, I, Ext> =
    Full<<Ext as Extra<'src, I>>::Error, <Ext as Extra<'src, I>>::State, <Ext as Extra<'src, I>>::Context>;

pub type State<S> = Full<EmptyErr, S, ()>;

pub type Context<C> = Full<EmptyErr, (), C>;

pub struct Full<E, S, C> {
    error: PhantomData<E>,
    pub(crate) state: S,
    pub(crate) context: C,
}

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

    fn new() -> ProvideExtra<'src, I, Self>
    where
        Self::State: Default,
        Self::Context: Default,
    {
        Full {
            error: PhantomData,
            state: Self::State::default(),
            context: Self::Context::default(),
        }
    }

    fn with_state(state: Self::State) -> ProvideExtra<'src, I, Self>
    where
        Self::Context: Default,
    {
        Full {
            error: PhantomData,
            state,
            context: Self::Context::default(),
        }
    }

    fn with_context(context: Self::Context) -> ProvideExtra<'src, I, Self>
    where
        Self::State: Default,
    {
        Full {
            error: PhantomData,
            state: Self::State::default(),
            context,
        }
    }

    fn with_state_and_context(state: Self::State, context: Self::Context) -> ProvideExtra<'src, I, Self> {
        Full {
            error: PhantomData,
            state,
            context,
        }
    }
}
