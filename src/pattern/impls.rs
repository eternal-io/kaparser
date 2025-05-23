use super::*;

pub struct Reiter<'s, 'p, 'i, U, E, P, S>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    S: AdvanceSlice<'i, U>,
{
    pub(super) body: &'p P,
    pub(super) src: &'s mut S,
    pub(super) phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, S> Iterator for Reiter<'_, '_, 'i, U, E, P, S>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    S: AdvanceSlice<'i, U>,
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

pub struct Joined<'s, 'p, 'i, U, E, P, Q, S>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
    S: AdvanceSlice<'i, U>,
{
    pub(super) body: &'p P,
    pub(super) sep: &'p Q,
    pub(super) src: &'s mut S,
    pub(super) end: bool,
    pub(super) phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, Q, S> Iterator for Joined<'_, '_, 'i, U, E, P, Q, S>
where
    U: ?Sized + Slice + 'i,
    E: Situation,
    P: Pattern<'i, U, E>,
    Q: Pattern<'i, U, E>,
    S: AdvanceSlice<'i, U>,
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
        let pat = (":", is_alpha..);
        let mut pat = pat.reiter(&mut sli);

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
