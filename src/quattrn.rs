use crate::{common::*, extra::*, input::*};

pub trait Quattrn<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type View<'tmp>
    where
        'src: 'tmp;

    #[doc(hidden)]
    fn advance<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        context: MaybeRef<Ext::Context>,
    ) -> Result<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp;

    fn lift<F, Out>(self, f: F)
    where
        Self: Sized,
        F: for<'all> Fn(Self::View<'all>) -> Out,
    {
        todo!()
    }
}
