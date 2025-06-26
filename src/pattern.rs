use crate::{extra::Extra, input::*, marker};

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

// pub trait Pattern<'src, U>
// where
//     U: Input<'src>,
// {
//     type Captured;

//     fn fullmatch(&self, input: &mut U) -> Self::Captured;
// }

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
