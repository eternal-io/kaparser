use super::*;
use core::mem;

pub struct AndThen<U, E, P, F, T>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(P::Captured) -> Result<T, E>,
    T: 'static + Clone,
{
    body: P,
    op: F,
    phantom: PhantomData<(U, E)>,
}
impl<U, E, P, F, T> Pattern<U, E> for AndThen<U, E, P, F, T>
where
    U: Slice,
    E: Situation,
    P: Pattern<U, E>,
    F: Fn(P::Captured) -> Result<T, E>,
    T: 'static + Clone,
{
    type Captured = T;
    type Internal = Alt3<P::Internal, (), T>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Alt3::Var2(())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        if !matches!(entry, Alt3::Var1(_)) {
            *entry = Alt3::Var1(self.body.init());
        }

        let Alt3::Var1(state) = entry else { unreachable!() };
        let offset = self.body.advance(slice, state, eof)?;

        let Alt3::Var1(state) = mem::replace(entry, Alt3::Var2(())) else {
            unreachable!()
        };

        *entry = Alt3::Var3((self.op)(self.body.extract(slice, state))?);

        Ok(offset)
    }
    #[inline]
    fn extract(&self, _lice: &U, entry: Self::Internal) -> Self::Captured {
        let Alt3::Var3(output) = entry else {
            panic!("contract violation")
        };
        output
    }
}
