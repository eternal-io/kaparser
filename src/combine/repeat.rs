use super::*;
use core::mem::MaybeUninit;

#[doc(inline)]
pub use crate::rep;

#[inline(always)]
pub const fn repeat<'i, U, E, P, const AT_LEAST: usize, const MAY_MORE: usize>(
    body: P,
) -> Repeat<'i, U, E, P, AT_LEAST, MAY_MORE>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    Repeat {
        body,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn repeat_exact<'i, U, E, P, const TIMES: usize>(body: P) -> RepeatExact<'i, U, E, P, TIMES>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    RepeatExact { body: repeat(body) }
}

#[inline(always)]
pub const fn repeat_at_most<'i, U, E, P, const TIMES: usize>(body: P) -> RepeatAtMost<'i, U, E, P, TIMES>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    RepeatAtMost { body: repeat(body) }
}

//------------------------------------------------------------------------------

pub struct Repeat<'i, U, E, P, const AT_LEAST: usize, const MAY_MORE: usize>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: P,
    phantom: PhantomData<(&'i U, E)>,
}

impl<'i, U, E, P, const AT_LEAST: usize, const MAY_MORE: usize> Pattern<'i, U, E>
    for Repeat<'i, U, E, P, AT_LEAST, MAY_MORE>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
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
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
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

                match self.body.advance(slice.split_at(*off).1, state, eof) {
                    Ok(len) => offset = *off + len,
                    Err(e) => match e.is_rejected() {
                        false => return e.raise_backtrack(*off),
                        true => match necessary {
                            true => return e.raise_backtrack(*off),
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

pub struct RepeatExact<'i, U, E, P, const TIMES: usize>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: Repeat<'i, U, E, P, TIMES, 0>,
}

impl<'i, U, E, P, const TIMES: usize> Pattern<'i, U, E> for RepeatExact<'i, U, E, P, TIMES>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = [P::Captured; TIMES];
    type Internal = <Repeat<'i, U, E, P, TIMES, 0> as Pattern<'i, U, E>>::Internal;

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
        self.body.extract(slice, entry).0
    }
}

//------------------------------------------------------------------------------

pub struct RepeatAtMost<'i, U, E, P, const TIMES: usize>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    body: Repeat<'i, U, E, P, 0, TIMES>,
}

impl<'i, U, E, P, const TIMES: usize> Pattern<'i, U, E> for RepeatAtMost<'i, U, E, P, TIMES>
where
    U: ?Sized + Slice,
    E: Situation,
    P: Pattern<'i, U, E>,
{
    type Captured = [Option<P::Captured>; TIMES];
    type Internal = <Repeat<'i, U, E, P, 0, TIMES> as Pattern<'i, U, E>>::Internal;

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
        self.body.extract(slice, entry).1
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn main() {
        let pat = simple_opaque(rep!(2..=4, [is_alpha]));
        assert_eq!(pat.full_match("z").unwrap_err().offset(), 1);
        assert_eq!(pat.full_match("zx").unwrap(), (['z', 'x'], [None, None]));
        assert_eq!(pat.full_match("zxc").unwrap(), (['z', 'x'], [Some('c'), None]));
        assert_eq!(pat.full_match("zxcv").unwrap(), (['z', 'x'], [Some('c'), Some('v')]));
        assert_eq!(pat.full_match("zxcvb").unwrap_err().offset(), 4);
        assert!(pat.parse(&mut "zxcvb").is_ok());

        assert_eq!(pat.full_match("z0").unwrap_err().offset(), 1);
        assert_eq!(pat.parse(&mut "zx0").unwrap(), (['z', 'x'], [None, None]));
        assert_eq!(pat.full_match("zx00").unwrap_err().offset(), 2);
    }
}
