use super::*;

#[inline(always)]
pub const fn cut<'i, U, P, Q>(head: P, body: Q) -> Cut<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    Q: Precede<'i, U>,
{
    Cut {
        head,
        body,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Cut<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    Q: Precede<'i, U>,
{
    head: P,
    body: Q,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, Q> Precede<'i, U> for Cut<'i, U, P, Q>
where
    U: 'i + ?Sized + Slice,
    P: Precede<'i, U>,
    Q: Precede<'i, U>,
{
    type Captured = (P::Captured, Q::Captured);
    type Internal = (usize, P::Internal, Q::Internal);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, self.head.init(), self.body.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let (cut, head, body) = entry;

        if *cut == 0 {
            let (t, len) = self.head.precede(slice, head, eof)?;
            if let Transfer::Accepted = t {
                *cut += len;
            } else {
                return Ok((t, len));
            }
        }

        let (t, len) = self.body.precede(slice.split_at(*cut).1, body, eof)?;
        if let Transfer::Accepted = t {
            Ok((Transfer::Accepted, len))
        } else {
            Ok((Transfer::Halt, len))
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (cut, head, body) = entry;
        let cap = self.head.extract(slice, head);
        let caq = self.body.extract(slice.split_at(cut).1, body);
        (cap, caq)
    }
}
