use crate::{common::*, error::*, extra::*, input::*, private};

pub trait Pattern<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type Captured;

    #[doc(hidden)]
    fn __fullmatch<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::Captured, I::Cursor), Ext::Error>;

    #[doc(hidden)]
    fn __flycheck<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>;

    //------------------------------------------------------------------------------

    fn fullmatch(&self, input: &mut I) -> PResult<Self::Captured, Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default,
    {
        self.fullmatch_with_state(input, &mut Ext::State::default())
    }

    fn fullmatch_with_state(&self, input: &mut I, state: &mut Ext::State) -> PResult<Self::Captured, Ext::Error>
    where
        Ext::Context: Default,
    {
        self.__fullmatch(
            input,
            input.begin(),
            state.into(),
            Ext::Context::default().into(),
            private::Token,
        )
        .verify_map(|(captured, cursor)| {
            (
                captured,
                (!input.has_reached_end(cursor.clone())).then(|| {
                    Ext::Error::new(
                        {
                            let off = input.offset(cursor);
                            off..off
                        },
                        ErrorKind::ExpectedEnd,
                    )
                }),
            )
        })
    }

    //------------------------------------------------------------------------------

    fn flycheck(&self, input: &mut I) -> Result<(), Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default,
    {
        self.flycheck_with_state(input, &mut Ext::State::default())
    }

    fn flycheck_with_state(&self, input: &mut I, state: &mut Ext::State) -> Result<(), Ext::Error>
    where
        Ext::Context: Default,
    {
        self.__flycheck(
            input,
            input.begin(),
            state.into(),
            Ext::Context::default().into(),
            private::Token,
        )
        .verify_map(|cursor| {
            (
                (),
                (!input.has_reached_end(cursor.clone())).then(|| {
                    Ext::Error::new(
                        {
                            let off = input.offset(cursor);
                            off..off
                        },
                        ErrorKind::ExpectedEnd,
                    )
                }),
            )
        })
        .into_result()
    }
}

// impl<'src, U, Q> Pattern<'src, U> for Q
// where
//     U::_Marker: marker::Static,
//     U: Input<'src>,
//     Q: Quattrn<'src, U>,
// {
//     type Captured = Q::View<'src>;

//     fn fullmatch(&self, input: &mut U) -> Self::Captured {
//         // SAFETY:
//         // This balnket implementation only works for inputs that marked as `StaticInput`,
//         // which ensures `'tmp` outlives `'src`, therefore the lifetime can be safely extended.
//         // In other words, they are inputs that do not need to be mutated when getting a slice or item.
//         unsafe {
//             core::mem::transmute(self.fullmatch_impl(input))
//             // Src = for<'tmp> Q::View<'tmp>;
//             // Dst = Q::View<'src>;
//         }
//     }
// }
