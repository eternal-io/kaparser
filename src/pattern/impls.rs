use super::*;

/// Note the generic parameter order.
pub const fn opaque<'i, U, P, E>(body: P) -> Opaque<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Opaque {
        body,
        phantom: PhantomData,
    }
}

pub const fn simple_opaque<'i, U, P>(body: P) -> Opaque<'i, U, SimpleError, P>
where
    U: ?Sized + Slice,
    P: Pattern<'i, U, SimpleError>,
{
    Opaque {
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Opaque<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P> Pattern<'i, U, E> for Opaque<'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = P::Captured;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        self.body.advance(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry)
    }
}

//------------------------------------------------------------------------------

pub struct Reiter<'p, 'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    pub(super) body: &'p P,
    pub(super) src: &'p mut &'i U,
    pub(super) phantom: PhantomData<E>,
}

impl<'i, U, E, P> Iterator for Reiter<'_, 'i, U, E, P>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Item = Result<P::Captured, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.body.parse(self.src) {
            Ok(cap) => Some(Ok(cap)),
            Err(e) => match e.is_halted() {
                true => Some(Err(e)),
                false => None,
            },
        }
    }
}

//------------------------------------------------------------------------------

pub struct Joined<'p, 'i, U, E, P, Q>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    pub(super) body: &'p P,
    pub(super) sep: &'p Q,
    pub(super) src: &'p mut &'i U,
    pub(super) end: bool,
    pub(super) phantom: PhantomData<E>,
}

impl<'i, U, E, P, Q> Iterator for Joined<'_, 'i, U, E, P, Q>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
{
    type Item = Result<(P::Captured, bool), E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        match self.body.parse(self.src) {
            Ok(cap) => match self.sep.parse(self.src) {
                Ok(_) => Some(Ok((cap, true))),
                Err(e) => match e.is_halted() {
                    true => Some(Err(e)),
                    false => {
                        self.end = true;
                        Some(Ok((cap, false)))
                    }
                },
            },
            Err(e) => match e.is_halted() {
                true => Some(Err(e)),
                false => None,
            },
        }
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn reiter() -> ParseResult<()> {
        let mut sli = ":qwer:uiop:zxcv:0000";
        let mut pat = (":", is_alpha..).reiter(&mut sli);

        assert_eq!(pat.next().unwrap()?, (":", "qwer"));
        assert_eq!(pat.next().unwrap()?, (":", "uiop"));
        assert_eq!(pat.next().unwrap()?, (":", "zxcv"));
        assert_eq!(sli, ":0000");

        Ok(())
    }

    #[test]
    fn joined() -> ParseResult<()> {
        let mut sli = "0123;;4567;;89AB;";
        let mut pat = (is_hex..).joined(&";;", &mut sli);

        assert_eq!(pat.next().unwrap()?, ("0123", true));
        assert_eq!(pat.next().unwrap()?, ("4567", true));
        assert_eq!(pat.next().unwrap()?, ("89AB", false));
        assert_eq!(sli, ";");

        Ok(())
    }
}
