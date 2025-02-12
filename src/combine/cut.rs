use super::*;

pub struct Cut<'i, U: ?Sized + Slice, P: Proceed<'i, U>> {
    pub(super) cut: P,
    phantom: PhantomData<&'i U>,
}
