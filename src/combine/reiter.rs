use super::*;

#[inline(always)]
pub const fn reiter<'i, U, P, R>(range: R, body: P) -> Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
{
    Reiterate {
        body,
        range,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn reiter_fold<'i, U, P, R, St, F>(range: R, f: F, body: P) -> ReiterateFold<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
    St: Default + Clone,
    F: Fn(&mut St, P::Captured),
{
    ReiterateFold {
        body,
        range,
        fold: f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
{
    body: P,
    range: R,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, R> Pattern<'i, U> for Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
{
    type Captured = &'i U;
    type Internal = (usize, usize, Option<P::Internal>);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1, None)
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (offset, times, state) = entry;
        while self.range.unfulfilled(*times) {
            let right = slice.split_at(*offset).1;
            let (t, len) = self
                .body
                .precede(right, state.get_or_insert_with(|| self.body.init()), eof)?;

            *offset += len;

            match t {
                Transfer::Accepted => drop(state.take()),
                Transfer::Rejected => break,
                Transfer::Halt => return Ok((t, len)),
            }

            *times += 1;
        }

        match self.range.contains(*times) {
            true => Ok((Transfer::Accepted, *offset)),
            false => Ok((Transfer::Rejected, *offset)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry.0).0
    }
}

//------------------------------------------------------------------------------

pub struct ReiterateFold<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
    St: Default + Clone,
    F: Fn(&mut St, P::Captured),
{
    body: P,
    range: R,
    fold: F,
    phantom: PhantomData<(&'i U, St)>,
}

impl<'i, U, P, R, St, F> Pattern<'i, U> for ReiterateFold<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    R: URangeBounds,
    St: 'static + Default + Clone,
    F: Fn(&mut St, P::Captured),
{
    type Captured = (&'i U, St);
    type Internal = (usize, usize, St, Option<P::Internal>);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1, St::default(), None)
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (offset, times, st, state) = entry;
        while self.range.unfulfilled(*times) {
            let right = slice.split_at(*offset).1;
            let (t, len) = self
                .body
                .precede(right, state.get_or_insert_with(|| self.body.init()), eof)?;

            *offset += len;

            match t {
                Transfer::Accepted => (self.fold)(st, self.body.extract(right, state.take().unwrap())),
                Transfer::Rejected => break,
                Transfer::Halt => return Ok((t, len)),
            }

            *times += 1;
        }

        match self.range.contains(*times) {
            true => Ok((Transfer::Accepted, *offset)),
            false => Ok((Transfer::Rejected, *offset)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (offset, _, st, _) = entry;
        (slice.split_at(offset).0, st)
    }
}
