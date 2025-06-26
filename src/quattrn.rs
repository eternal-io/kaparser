use crate::{extra::*, input::*};

pub trait Quattrn<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type View<'tmp>
    where
        'src: 'tmp;

    fn fullmatch_impl<'tmp>(&self, input: &'tmp mut I) -> Self::View<'tmp>
    where
        'src: 'tmp;
}
