use crate::{common::*, extra::*, input::*, parser::*, pattern::*, private};
use core::marker::PhantomData;

pub struct Captured<P> {
    pub(crate) pattern: P,
}

impl<'src, I, Ext, P> Parser<'src, I, P::View<'src>, Ext> for Captured<P>
where
    I: Input<'src> + StaticInput,
    Ext: Extra<'src, I>,
    P: Pattern<'src, I, Ext>,
{
    fn __parse(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(P::View<'src>, I::Cursor), Ext::Error> {
        self.pattern
            .__parse(input, start, state, ctx, private::Token)
            .raise_or_map(|(view, cur)| {
                // SAFETY:
                // This converter only works for inputs that marked as `StaticInput`,
                // which ensures `'tmp` outlives `'src`, therefore the lifetime can be safely extended.
                (unsafe { core::mem::transmute::<P::View<'_>, P::View<'src>>(view) }, cur)
            })
    }

    __forward_check!(pattern);
}

//------------------------------------------------------------------------------

pub struct Lift<P, F, Out> {
    pub(crate) pattern: P,
    pub(crate) mapper: F,
    pub(crate) phantom: PhantomData<Out>,
}

impl<'src, I, Ext, P, F, O> Parser<'src, I, O, Ext> for Lift<P, F, O>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
    P: Pattern<'src, I, Ext>,
    F: for<'all> Fn(P::View<'all>) -> O,
{
    fn __parse(
        &self,
        input: &mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(O, I::Cursor), Ext::Error> {
        self.pattern
            .__parse(input, start, state, ctx, private::Token)
            .raise_or_map(|(view, cur)| ((self.mapper)(view), cur))
    }

    __forward_check!(pattern);
}

//------------------------------------------------------------------------------
