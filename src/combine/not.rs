use super::*;

#[inline(always)]
pub const fn not<T, P: Predicate2<T>>(predicate: P) -> Not<T, P> {
    Not(predicate, PhantomData)
}

//------------------------------------------------------------------------------

pub struct Not<T, P: Predicate2<T>>(P, PhantomData<T>);

impl<T, P: Predicate2<T>> Predicate2<T> for Not<T, P> {
    #[inline(always)]
    fn predicate(&self, value: T) -> bool {
        !self.0.predicate(value)
    }
}
