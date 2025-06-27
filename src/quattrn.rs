use crate::{extra::*, input::*};

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
        extra: &mut ProvideExtra<'src, I, Ext>,
    ) -> Result<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp;
}
