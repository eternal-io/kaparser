use super::*;

// TODO: prefix, suffix, enclose

//------------------------------------------------------------------------------

pub struct Prefix<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    Q: Proceed<'i, U>,
{
    body: P,
    prefix: Q,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, Q> Proceed<'i, U> for Prefix<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
    Q: Proceed<'i, U>,
{
    type Captured = P::Captured;
    type Internal = (usize, Alt2<P::Internal, Q::Internal>);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, Alt2::Var2(self.prefix.init()))
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        let (offset, state) = entry;

        if let Alt2::Var2(prefix) = state {
            let (t, len) = self.prefix.proceed(slice, prefix, eof)?;

            *offset += len;

            match t {
                Transfer::Accepted => (),
                t => return Ok((t, len)),
            }

            *state = Alt2::Var1(self.body.init());
        }

        let Alt2::Var1(body) = state else { unreachable!() };
        let (t, len) = self.body.proceed(slice.split_at(*offset).1, body, eof)?;

        *offset += len;

        Ok((t, len))
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (offset, state) = entry;
        let Alt2::Var1(body) = state else { unreachable!() };
        self.body.extract(slice.split_at(offset).1, body)
    }
}

//------------------------------------------------------------------------------
