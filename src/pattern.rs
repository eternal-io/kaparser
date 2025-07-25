use crate::{common::*, converter, extra::*, input::*, parser::*, private};
use core::marker::PhantomData;

pub trait Pattern<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type View<'tmp>
    where
        'src: 'tmp;

    #[doc(hidden)]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp;

    #[doc(hidden)]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp;

    //------------------------------------------------------------------------------

    fn captured(self) -> impl Parser<'src, I, Self::View<'src>, Ext>
    where
        Self: Sized,
        I: StaticInput,
    {
        converter::Captured { pattern: self }
    }

    fn lift<F, O>(self, mapper: F) -> impl Parser<'src, I, O, Ext>
    where
        Self: Sized,
        F: for<'all> Fn(Self::View<'all>) -> O,
    {
        converter::Lift {
            pattern: self,
            mapper,
            phantom: PhantomData,
        }
    }
}
