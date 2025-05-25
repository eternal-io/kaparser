use super::*;

#[inline]
pub const fn peek<P>(body: P) -> Peek<P> {
    Peek { body }
}

//------------------------------------------------------------------------------

pub struct Peek<P> {
    body: P,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Peek<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof).map(|_| 0)
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_peek() {
        let pat_next = opaque_simple("foo");
        let pat_peek = opaque_simple(peek("foo"));

        let slice = &mut "foobar";
        assert_eq!(pat_next.parse(slice).unwrap(), "foo");
        assert_eq!(*slice, "bar");

        let slice = &mut "foobar";
        assert_eq!(pat_peek.parse(slice).unwrap(), "foo");
        assert_eq!(*slice, "foobar");
    }
}
