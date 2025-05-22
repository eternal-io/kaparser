use super::*;

#[inline]
pub const fn not<T, P: Predicate<T>>(predicate: P) -> Not<T, P> {
    Not(predicate, PhantomData)
}

//------------------------------------------------------------------------------

pub struct Not<T, P: Predicate<T>>(P, PhantomData<T>);

impl<T, P: Predicate<T>> Predicate<T> for Not<T, P> {
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        !self.0.predicate(item)
    }
}
