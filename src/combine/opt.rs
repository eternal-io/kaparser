use super::*;

#[inline(always)]
pub const fn opt<'i, U, E, P>(opt: P) -> Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Optional {
        opt,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    opt: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Optional<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = Option<P::Captured>;
    type Internal = Option<P::Internal>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        Some(self.opt.init())
    }
    #[inline(always)]
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
    #[inline(always)]
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
        let pat = simple_opaque(opt(is_alpha..));
        assert_eq!(pat.full_match("qwer").unwrap(), Some("qwer"));
        assert_eq!(pat.full_match("7890").unwrap_err().offset(), 0);
        assert_eq!(pat.parse(&mut "7890").unwrap(), None);
        assert_eq!(pat.parse(&mut "LB90").unwrap(), Some("LB"));

        let pat = simple_opaque(opt(..=[is_ctrl]));
        assert_eq!(pat.full_match("xyz\n").unwrap(), Some(("xyz", '\n')));
        assert_eq!(pat.full_match("xyzww").unwrap_err().offset(), 5);
    }
}
