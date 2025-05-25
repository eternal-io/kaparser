use super::*;

#[inline]
pub const fn opt<P>(opt: P) -> Optional<P> {
    Optional { opt }
}

//------------------------------------------------------------------------------

pub struct Optional<P> {
    opt: P,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Optional<P>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline]
    fn init(&self) -> Self::Internal {
        Some(self.opt.init())
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let res = self.opt.advance(slice, entry.as_mut().unwrap(), eof);
        if let Err(e) = &res {
            if e.is_rejected() {
                *entry = None;
                return Ok(0);
            }
        }
        res
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        entry.map(|state| self.opt.extract(slice, state))
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn main() {
        let pat = opaque_simple(opt(is_alpha..));
        assert_eq!(pat.fullmatch("qwer").unwrap(), Some("qwer"));
        assert_eq!(pat.fullmatch("7890").unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut "7890").unwrap(), None);
        assert_eq!(pat.parse(&mut "LB90").unwrap(), Some("LB"));

        let pat = opaque_simple(opt(..=[is_ctrl]));
        assert_eq!(pat.fullmatch("xyz\n").unwrap(), Some(("xyz", '\n')));
        assert_eq!(pat.fullmatch("xyzww").unwrap_err().offset(), 5);
    }
}
