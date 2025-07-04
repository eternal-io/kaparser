use crate::{common::*, extra::*, input::*, parser::*, pattern::*, private};
use core::marker::PhantomData;

pub struct Captured<Q> {
    pub(crate) quattrn: Q,
}

impl<'src, I, Ext, Q> Parser<'src, I, Ext> for Captured<Q>
where
    I: Input<'src> + StaticInput,
    Ext: Extra<'src, I>,
    Q: Pattern<'src, I, Ext>,
{
    type Output = Q::View<'src>;

    fn __parse(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::Output, I::Cursor), Ext::Error> {
        let PResult { value, error } = self.quattrn.__parse(input, start, state, ctx, private::Token);

        PResult {
            value: value.map(|(view, cur)| {
                // SAFETY:
                // This converter only works for inputs that marked as `StaticInput`,
                // which ensures `'tmp` outlives `'src`, therefore the lifetime can be safely extended.
                // In other words, they are inputs that do not need to be mutated when getting a slice or item.
                (unsafe { core::mem::transmute(view) }, cur)
            }),
            error,
        }
    }

    fn __check(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error> {
        self.quattrn.__check(input, start, state, ctx, private::Token)
    }
}

//------------------------------------------------------------------------------

pub struct Lift<Q, F, Out> {
    pub(crate) quattrn: Q,
    pub(crate) mapper: F,
    pub(crate) phantom: PhantomData<Out>,
}

impl<'src, I, Ext, Q, F, Out> Parser<'src, I, Ext> for Lift<Q, F, Out>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
    Q: Pattern<'src, I, Ext>,
    F: for<'all> Fn(Q::View<'all>) -> Out,
{
    type Output = Out;

    fn __parse(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::Output, I::Cursor), Ext::Error> {
        let PResult { value, error } = self.quattrn.__parse(input, start, state, ctx, private::Token);

        PResult {
            value: value.map(|(view, cur)| ((self.mapper)(view), cur)),
            error,
        }
    }

    fn __check(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error> {
        self.quattrn.__check(input, start, state, ctx, private::Token)
    }
}

//------------------------------------------------------------------------------
