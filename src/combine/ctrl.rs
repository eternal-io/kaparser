use super::*;

#[inline(always)]
pub const fn cond<U, P>(b: bool, body: P) -> Conditional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    Conditional {
        cond: b,
        body,
        phantom: PhantomData,
    }
}

// #[inline(always)]
// #[doc(alias = "filter")]
// pub const fn verify<'i: 'j, 'j, U, P, F>(f: F, body: P) -> Verify<'j, U, P, F>
// where
//     U: Slice2,
//     P: Pattern2<'j, U>,
//     F: Fn(P::Captured) -> Transfer,
// {
//     Verify {
//         body,
//         verify: f,
//         phantom: PhantomData,
//     }
// }

//------------------------------------------------------------------------------

pub struct Conditional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    cond: bool,
    body: P,
    phantom: PhantomData<U>,
}

impl<U, P> Pattern2<U> for Conditional<U, P>
where
    U: Slice2,
    P: Pattern2<U>,
{
    type Captured = Option<P::Captured>;
    type Internal = P::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
        match self.cond {
            true => self.body.precede2(slice, entry, eof),
            false => Some((Transfer::Accepted, 0)),
        }
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        match self.cond {
            true => Some(self.body.extract2(slice, entry)),
            false => None,
        }
    }
}

//------------------------------------------------------------------------------

// pub struct Verify<'i: 'j, 'j, U, P, F>
// where
//     U: Slice2,
//     P: Pattern2<'j, U>,
//     F: Fn(P::Captured) -> Transfer,
// {
//     body: P,
//     verify: F,
//     phantom: PhantomData<(U, &'j ())>,
// }

// impl<'i: 'j, 'j, U, P, F> Pattern2<U> for Verify<'j, U, P, F>
// where
//     U: Slice2,
//     P: Pattern2<'j, U>,
//     F: Fn(P::Captured) -> Transfer,
// {
//     type Captured = P::Captured;
//     type Internal = P::Internal;

//     #[inline(always)]
//     fn init(&self) -> Self::Internal {
//         self.body.init()
//     }
//     #[inline(always)]
//     #[allow(unsafe_code)]
//     fn precede(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Option<(Transfer, usize)> {
//         let (t, len) = self.body.precede(slice, entry, eof)?;
//         if !t.is_accepted() {
//             return Some((t, len));
//         }

//         Some((
//             (self.verify)(self.body.extract(
//                 unsafe {
//                     // Safety: Extend lifetime. (TODO: Need more clarification.)
//                     // Guaranteed not to bring dangling references, because `&U` is already outlives `'j` (`'i: 'j`).
//                     ::core::mem::transmute::<&U, &U>(slice)
//                 },
//                 entry.clone(),
//             )),
//             len,
//         ))
//     }
//     #[inline(always)]
//     fn extract(&self, slice: U, entry: Self::Internal) -> Self::Captured {
//         self.body.extract(slice, entry)
//     }
// }
