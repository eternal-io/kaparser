use super::*;

#[doc(inline)]
pub use crate::rep;

#[inline(always)]
pub const fn repeat<'i, U, P, const AT_LEAST: usize, const MAY_MORE: usize>(
    body: P,
) -> Repeat<'i, U, P, AT_LEAST, MAY_MORE>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    Repeat {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn repeat_exact<'i, U, P, const TIMES: usize>(body: P) -> RepeatExact<'i, U, P, TIMES>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    RepeatExact { body: repeat(body) }
}

#[inline(always)]
pub const fn repeat_at_most<'i, U, P, const TIMES: usize>(body: P) -> RepeatAtMost<'i, U, P, TIMES>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    RepeatAtMost { body: repeat(body) }
}

//------------------------------------------------------------------------------

pub struct Repeat<'i, U, P, const AT_LEAST: usize, const MAY_MORE: usize>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    body: P,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, const AT_LEAST: usize, const MAY_MORE: usize> Proceed<'i, U> for Repeat<'i, U, P, AT_LEAST, MAY_MORE>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    type Captured = ([P::Captured; AT_LEAST], [Option<P::Captured>; MAY_MORE]);
    type Internal = (
        usize,
        [(usize, P::Internal); AT_LEAST],
        [(usize, P::Internal); MAY_MORE],
    );

    #[inline(always)]
    #[allow(unsafe_code)]
    fn init(&self) -> Self::Internal {
        let mut at_least: MaybeUninit<[(usize, P::Internal); AT_LEAST]> = MaybeUninit::uninit();
        let mut may_more: MaybeUninit<[(usize, P::Internal); MAY_MORE]> = MaybeUninit::uninit();
        let item = self.body.init();

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
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
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

                let (t, len) = self.body.proceed(slice.split_at(*off).1, state, eof)?;
                offset = *off + len;
                match t {
                    Transfer::Accepted => (),

                    Transfer::Rejected => match necessary {
                        true => return Ok((Transfer::Rejected, offset)),
                        false => break,
                    },
                    Transfer::Halt => {
                        cold_path();
                        return Ok((Transfer::Halt, offset));
                    }
                }

                *checkpoint += 1;
            }
        }

        Ok((Transfer::Accepted, offset))
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (mut checkpoint, at_least, may_more) = entry;
        let mut at_least_cap: MaybeUninit<[P::Captured; AT_LEAST]> = MaybeUninit::uninit();
        let mut may_more_cap: MaybeUninit<[Option<P::Captured>; MAY_MORE]> = MaybeUninit::uninit();

        for (i, (off, state)) in at_least.into_iter().enumerate() {
            checkpoint -= 1;
            let cap = self.body.extract(slice.split_at(off).1, state);
            unsafe {
                (&raw mut (*at_least_cap.as_mut_ptr())[i]).write(cap);
            }
        }
        for (i, (off, state)) in may_more.into_iter().enumerate() {
            if checkpoint > 0 {
                checkpoint -= 1;
                let cap = self.body.extract(slice.split_at(off).1, state);
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

pub struct RepeatExact<'i, U, P, const TIMES: usize>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    body: Repeat<'i, U, P, TIMES, 0>,
}

impl<'i, U, P, const TIMES: usize> Proceed<'i, U> for RepeatExact<'i, U, P, TIMES>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    type Captured = [P::Captured; TIMES];
    type Internal = <Repeat<'i, U, P, TIMES, 0> as Proceed<'i, U>>::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        self.body.proceed(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry).0
    }
}

//------------------------------------------------------------------------------

pub struct RepeatAtMost<'i, U, P, const TIMES: usize>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    body: Repeat<'i, U, P, 0, TIMES>,
}

impl<'i, U, P, const TIMES: usize> Proceed<'i, U> for RepeatAtMost<'i, U, P, TIMES>
where
    U: 'i + ?Sized + Slice,
    P: Proceed<'i, U>,
{
    type Captured = [Option<P::Captured>; TIMES];
    type Internal = <Repeat<'i, U, P, 0, TIMES> as Proceed<'i, U>>::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.body.init()
    }
    #[inline(always)]
    fn proceed(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> ProceedResult {
        self.body.proceed(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        self.body.extract(slice, entry).1
    }
}
