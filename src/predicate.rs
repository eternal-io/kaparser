pub trait Predicate<T>: Sized {
    fn predicate(&self, item: &T) -> bool;
}
