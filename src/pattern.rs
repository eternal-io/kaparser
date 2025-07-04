use crate::{common::*, extra::*, input::*, private};

pub trait Pattern<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type Output;

    #[doc(hidden)]
    fn __parse(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::Output, I::Cursor), Ext::Error>;

    #[doc(hidden)]
    fn __check(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>;

    //------------------------------------------------------------------------------

    fn parse(&self, input: &mut I, start: I::Cursor) -> PResult<(Self::Output, I::Cursor), Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default,
    {
        self.parse_with_state(input, start, &mut Ext::State::default())
    }

    fn parse_with_state(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: &mut Ext::State,
    ) -> PResult<(Self::Output, I::Cursor), Ext::Error>
    where
        Ext::Context: Default,
    {
        self.__parse(
            input,
            start,
            state.into(),
            Ext::Context::default().into(),
            private::Token,
        )
    }

    fn fullmatch(&self, input: &mut I) -> PResult<Self::Output, Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default,
    {
        self.fullmatch_with_state(input, &mut Ext::State::default())
    }

    fn fullmatch_with_state(&self, input: &mut I, state: &mut Ext::State) -> PResult<Self::Output, Ext::Error>
    where
        Ext::Context: Default,
    {
        self.__parse(
            input,
            input.begin(),
            state.into(),
            Ext::Context::default().into(),
            private::Token,
        )
        .verify_map(|(cap, cur)| (cap, input.shall_reached_end(cur)))
    }

    fn fullcheck(&self, input: &mut I) -> Result<(), Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default,
    {
        self.fullcheck_with_state(input, &mut Ext::State::default())
    }

    fn fullcheck_with_state(&self, input: &mut I, state: &mut Ext::State) -> Result<(), Ext::Error>
    where
        Ext::Context: Default,
    {
        self.__check(
            input,
            input.begin(),
            state.into(),
            Ext::Context::default().into(),
            private::Token,
        )
        .verify_map(|cur| ((), input.shall_reached_end(cur)))
        .into_result()
    }

    //------------------------------------------------------------------------------
}
