use super::*;
use core::num::NonZeroUsize;

//------------------------------------------------------------------------------

#[doc(hidden)]
#[derive(Clone)]
pub enum Phase {
    PrimaryLeft,
    SecondaryLeft,
    SecondaryRight,
    PrimaryRight,
}

//------------------------------------------------------------------------------

pub struct Winged<'i, T> {
    primary: T,
    secondary: T,
    phantom: PhantomData<&'i ()>,
}

impl<'i> Proceed<'i, str> for Winged<'i, char> {
    type Captured = &'i str;
    type Internal = (usize, Phase, usize, usize);

    fn init(&self) -> Self::Internal {
        (0, Phase::PrimaryLeft, 0, 0)
    }

    fn proceed(&self, slice: &'i str, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let (tot_off, phase, content_start, content_len) = entry;
        proceed! {
            phase => {
                'A: Phase::PrimaryLeft => {}
                'B: Phase::SecondaryLeft => {}
                'C: Phase::SecondaryRight => {}
                'D: Phase::PrimaryRight => {}
            }
        }

        todo!()
    }

    fn extract(&self, slice: &'i str, entry: Self::Internal) -> Self::Captured {
        todo!()
    }
}
