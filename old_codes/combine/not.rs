use super::*;

#[inline]
pub const fn not<P>(predicate: P) -> Not<P> {
    Not(predicate)
}

//------------------------------------------------------------------------------

pub struct Not<P>(P);

impl<T, P> Predicate<T> for Not<P>
where
    P: Predicate<T>,
{
    #[inline]
    fn predicate(&self, item: &T) -> bool {
        !self.0.predicate(item)
    }
}
