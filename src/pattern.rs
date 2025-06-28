use crate::{extra::Extra, input::*, marker};

pub trait Pattern<'src, I, Ext>
where
    I: Input<'src>,
    Ext: Extra<'src, I>,
{
    type Captured;

    // TODO: Custom result type due to here are non-fatal errors.
    fn fullmatch(&self, input: &mut I) -> Result<Self::Captured, Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default;

    fn fullmatch_with_state(&self, input: &mut I, state: &'src mut Ext::State) -> Result<Self::Captured, Ext::Error>
    where
        Ext::Context: Default;

    fn flycheck(&self, input: &mut I) -> Result<(), Ext::Error>
    where
        Ext::State: Default,
        Ext::Context: Default;

    fn flycheck_with_state(&self, input: &mut I, state: &'src mut Ext::State) -> Result<(), Ext::Error>
    where
        Ext::Context: Default;
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
