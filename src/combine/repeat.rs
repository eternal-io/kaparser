use super::*;

#[doc(inline)]
pub use crate::rep;

#[inline(always)]
pub const fn repeat<U, E, P, const AT_LEAST: usize, const MAY_MORE: usize>(
    body: P,
) -> Repeat<U, E, P, AT_LEAST, MAY_MORE>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    Repeat {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn repeat_exact<U, E, P, const TIMES: usize>(body: P) -> RepeatExact<U, E, P, TIMES>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    RepeatExact { body: repeat(body) }
}

#[inline(always)]
pub const fn repeat_at_most<U, E, P, const TIMES: usize>(body: P) -> RepeatAtMost<U, E, P, TIMES>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    RepeatAtMost { body: repeat(body) }
}

//------------------------------------------------------------------------------

pub struct Repeat<U, E, P, const AT_LEAST: usize, const MAY_MORE: usize>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    body: P,
    phantom: PhantomData<(U, E)>,
}

impl<U, E, P, const AT_LEAST: usize, const MAY_MORE: usize> Pattern2<U, E> for Repeat<U, E, P, AT_LEAST, MAY_MORE>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    type Captured = ([P::Captured; AT_LEAST], [Option<P::Captured>; MAY_MORE]);
    type Internal = (
        usize,
        [(usize, P::Internal); AT_LEAST],
        [(usize, P::Internal); MAY_MORE],
    );

    #[inline(always)]
    #[allow(unsafe_code)]
    fn init2(&self) -> Self::Internal {
        let mut at_least: MaybeUninit<[(usize, P::Internal); AT_LEAST]> = MaybeUninit::uninit();
        let mut may_more: MaybeUninit<[(usize, P::Internal); MAY_MORE]> = MaybeUninit::uninit();
        let item = self.body.init2();

        for i in 0..AT_LEAST {
            unsafe {
                (&raw mut (*at_least.as_mut_ptr())[i]).write((0, item.clone()));
            }
        }
        for i in 0..MAY_MORE {
            unsafe {
                (&raw mut (*may_more.as_mut_ptr())[i]).write((0, item.clone()));
            }
        }

        unsafe { (0, at_least.assume_init(), may_more.assume_init()) }
    }

    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        let (checkpoint, at_least, may_more) = entry;
        let mut resuming = *checkpoint;
        let mut offset = 0usize;

        for (children, necessary) in [(at_least.iter_mut(), true), (may_more.iter_mut(), false)].into_iter() {
            for (off, state) in children {
                if unlikely(resuming > 0) {
                    resuming -= 1;
                    continue;
                }

                if likely(*off == 0) {
                    *off = offset;
                }

                match self.body.precede2(slice.split_at(*off).1, state, eof) {
                    Ok(len) => offset = *off + len,
                    Err(e) => match e.is_rejected() {
                        false => return Err(e), // FIXME: Error location.
                        true => match necessary {
                            true => return Err(e),
                            false => break,
                        },
                    },
                }

                *checkpoint += 1;
            }
        }

        Ok(offset)
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        let (mut checkpoint, at_least, may_more) = entry;
        let mut at_least_cap: MaybeUninit<[P::Captured; AT_LEAST]> = MaybeUninit::uninit();
        let mut may_more_cap: MaybeUninit<[Option<P::Captured>; MAY_MORE]> = MaybeUninit::uninit();

        for (i, (off, state)) in at_least.into_iter().enumerate() {
            checkpoint -= 1;
            let cap = self.body.extract2(slice.split_at(off).1, state);
            unsafe {
                (&raw mut (*at_least_cap.as_mut_ptr())[i]).write(cap);
            }
        }
        for (i, (off, state)) in may_more.into_iter().enumerate() {
            if checkpoint > 0 {
                checkpoint -= 1;
                let cap = self.body.extract2(slice.split_at(off).1, state);
                unsafe {
                    (&raw mut (*may_more_cap.as_mut_ptr())[i]).write(Some(cap));
                }
            } else {
                unsafe {
                    (&raw mut (*may_more_cap.as_mut_ptr())[i]).write(None);
                }
            }
        }

        unsafe { (at_least_cap.assume_init(), may_more_cap.assume_init()) }
    }
}

//------------------------------------------------------------------------------

pub struct RepeatExact<U, E, P, const TIMES: usize>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    body: Repeat<U, E, P, TIMES, 0>,
}

impl<U, E, P, const TIMES: usize> Pattern2<U, E> for RepeatExact<U, E, P, TIMES>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    type Captured = [P::Captured; TIMES];
    type Internal = <Repeat<U, E, P, TIMES, 0> as Pattern2<U, E>>::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body.precede2(slice, entry, eof)
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract2(slice, entry).0
    }
}

//------------------------------------------------------------------------------

pub struct RepeatAtMost<U, E, P, const TIMES: usize>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    body: Repeat<U, E, P, 0, TIMES>,
}

impl<U, E, P, const TIMES: usize> Pattern2<U, E> for RepeatAtMost<U, E, P, TIMES>
where
    U: Slice2,
    E: Situation,
    P: Pattern2<U, E>,
{
    type Captured = [Option<P::Captured>; TIMES];
    type Internal = <Repeat<U, E, P, 0, TIMES> as Pattern2<U, E>>::Internal;

    #[inline(always)]
    fn init2(&self) -> Self::Internal {
        self.body.init2()
    }
    #[inline(always)]
    fn precede2(&self, slice: U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult<E> {
        self.body.precede2(slice, entry, eof)
    }
    #[inline(always)]
    fn extract2(&self, slice: U, entry: Self::Internal) -> Self::Captured {
        self.body.extract2(slice, entry).1
    }
}
