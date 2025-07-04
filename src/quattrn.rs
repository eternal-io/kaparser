use crate::{common::*, converter, extra::*, input::*, pattern::*, private};
use core::marker::PhantomData;

pub trait Quattrn<'src, I, Ext>
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

    fn captured(self) -> impl Pattern<'src, I, Ext, Output = Self::View<'src>>
    where
        Self: Sized,
        I: StaticInput,
    {
        converter::Captured { quattrn: self }
    }

    fn lift<F, Out>(self, mapper: F) -> impl Pattern<'src, I, Ext, Output = Out>
    where
        Self: Sized,
        F: for<'all> Fn(Self::View<'all>) -> Out,
    {
        converter::Lift {
            quattrn: self,
            mapper,
            phantom: PhantomData,
        }
    }
}
