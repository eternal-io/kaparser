use super::*;

#[inline(always)]
pub const fn reiterate<'i, U, P, R>(range: R, body: P) -> Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    R: URangeBounds,
{
    Reiterate {
        body,
        range,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn reiterate_with<'i, U, P, R, St, F>(range: R, fold: F, body: P) -> ReiterateWith<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    R: URangeBounds,
    St: Default + Clone,
    F: Fn(&mut St, P::Captured),
{
    ReiterateWith {
        body,
        range,
        fold,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    R: URangeBounds,
{
    body: P,
    range: R,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, R> Proceed<'i, U> for Reiterate<'i, U, P, R>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    R: URangeBounds,
{
    type Captured = &'i U;
    type Internal = (usize, usize, Option<P::Internal>);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 1, None)
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let (tot_off, times, state) = entry;
        while self.range.unfulfilled(*times) {
            let right = slice.split_at(*tot_off).1;
            let (t, len) = self
                .body
                .proceed(right, state.get_or_insert_with(|| self.body.init()), eof)?;

            *tot_off += len;

            match t {
                Transfer::Accepted => drop(state.take()),
                Transfer::Rejected => break,
                Transfer::Halt => return Ok((Transfer::Halt, len)),
            }

            *times += 1;
        }

        match self.range.contains(*times) {
            true => Ok((Transfer::Accepted, *tot_off)),
            false => Ok((Transfer::Rejected, *tot_off)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        slice.split_at(entry.0).0
    }
}

//------------------------------------------------------------------------------

pub struct ReiterateWith<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    R: URangeBounds,
    St: Default + Clone,
    F: Fn(&mut St, P::Captured),
{
    body: P,
    range: R,
    fold: F,
    phantom: PhantomData<(&'i U, St)>,
}

impl<'i, U, P, R, St, F> Proceed<'i, U> for ReiterateWith<'i, U, P, R, St, F>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
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
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let (tot_off, times, st, state) = entry;
        while self.range.unfulfilled(*times) {
            let right = slice.split_at(*tot_off).1;
            let (t, len) = self
                .body
                .proceed(right, state.get_or_insert_with(|| self.body.init()), eof)?;

            *tot_off += len;

            match t {
                Transfer::Accepted => (self.fold)(st, self.body.extract(right, state.take().unwrap())),
                Transfer::Rejected => break,
                Transfer::Halt => return Ok((Transfer::Halt, len)),
            }

            *times += 1;
        }

        match self.range.contains(*times) {
            true => Ok((Transfer::Accepted, *tot_off)),
            false => Ok((Transfer::Rejected, *tot_off)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (tot_off, _, st, _) = entry;
        (slice.split_at(tot_off).0, st)
    }
}
