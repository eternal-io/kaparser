use crate::{common::*, error::*, extra::*, input::*, pattern::*, predicate::*, private, slice::*};
use core::{fmt, marker::PhantomData, mem::MaybeUninit};

struct Single<'a, Token, Pred>(&'a Pred, PhantomData<Token>);

impl<'a, Token, Pred> Describe for Single<'a, Token, Pred>
where
    Pred: Predicate<Token>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a single token matches ")?;
        self.0.describe(f)
    }
}

impl<'src, I, Ext, Pred> Pattern<'src, I, Ext> for [Pred; 1]
where
    I: InputOwnableToken<'src>,
    Ext: Extra<'src, I>,
    Pred: Predicate<I::Token>,
{
    type View<'tmp>
        = I::Token
    where
        'src: 'tmp;

    #[inline]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        Pattern::<I, Ext>::__check(self, input, start.clone(), state, ctx, private::Token)
            .raise_or_map(|end| (input.get_owned(start).unwrap(), end))
    }

    #[inline]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp,
    {
        drop((state, ctx));
        let mut end = start.clone();
        PResult::from(input.next_maybe_ref::<Ext::Error>(&mut end))
            .flatten()
            .raise_or_and_then(|token| match token.verify_by(&self[0]) {
                true => Ok(end),
                false => Err(Ext::Error::new(
                    I::span(start..end),
                    ErrorKind::Expected(&Single(&self[0], PhantomData)),
                )),
            })
    }
}

//------------------------------------------------------------------------------

pub struct Ref<Pred>(pub Pred);

impl<'src, I, Ext, Pred> Pattern<'src, I, Ext> for Ref<Pred>
where
    I: InputBorrowableToken<'src>,
    Ext: Extra<'src, I>,
    Pred: Predicate<I::Token>,
{
    type View<'tmp>
        = &'tmp I::Token
    where
        'src: 'tmp;

    #[inline]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        Pattern::<I, Ext>::__check(self, input, start.clone(), state, ctx, private::Token)
            .raise_or_map(|end| (input.get_borrowed(start).unwrap(), end))
    }

    #[inline]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp,
    {
        drop((state, ctx));
        let mut end = start.clone();
        PResult::from(input.next_maybe_ref::<Ext::Error>(&mut end))
            .flatten()
            .raise_or_and_then(|token| match token.verify_by(&self.0) {
                true => Ok(end),
                false => Err(Ext::Error::new(
                    I::span(start..end),
                    ErrorKind::Expected(&Single(&self.0, PhantomData)),
                )),
            })
    }
}

//------------------------------------------------------------------------------

pub struct Take<Token, Pred, R> {
    pub(crate) pred: Pred,
    pub(crate) range: R,
    pub(crate) phantom: PhantomData<Token>,
}

impl<Token, Pred, R> Describe for Take<Token, Pred, R>
where
    Pred: Predicate<Token>,
    R: URangeBounds,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.range.describe(f)?;
        write!(f, " tokens matches ")?;
        self.pred.describe(f)
    }
}

impl<'src, I, Ext, Pred, R> Pattern<'src, I, Ext> for Take<I::Token, Pred, R>
where
    I: InputSlice<'src>,
    Ext: Extra<'src, I>,
    Pred: Predicate<I::Token>,
    R: URangeBounds,
{
    type View<'tmp>
        = &'tmp I::Slice
    where
        'src: 'tmp;

    #[inline]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &self.range, input, start.clone())
            .raise_or_map(|end| (input.release_slice(start..end.clone()), end))
    }

    #[inline]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &self.range, input, start)
    }
}

#[inline]
fn __check_take<'src, Desc, Pred, R, I, E>(
    desc: &Desc,
    pred: &Pred,
    range: &R,
    input: &mut I,
    start: I::Cursor,
) -> PResult<I::Cursor, E>
where
    Desc: Describe,
    Pred: Predicate<I::Token>,
    R: URangeBounds,
    I: InputSlice<'src>,
    E: Error,
{
    let mut times = 0;
    let mut offset = 0;
    let end = loop {
        let (slice, eof) = trip!(input.fetch_slice(start.clone()));

        if I::Slice::ITEM_HAS_FIXED_LENGTH_1 && slice.len() < range.lower_bound() {
            match eof {
                true => break I::bump_cursor(start.clone(), slice.len()),
                false => continue,
            }
        }

        if let Some((i, (off, item))) = slice
            .after(offset)
            .iter_indices()
            .enumerate()
            .take_while(|(i, (_off, item))| range.unfulfilled(times + *i) && item.verify_by(pred))
            .last()
        {
            times += i + 1;
            offset += off + I::Slice::len_of(item.as_ref());
        }

        let end = I::bump_cursor(start.clone(), offset);

        if !eof && range.unfulfilled(times) {
            continue;
        } else if range.contains(times) {
            return PResult::emit(end);
        } else {
            break end;
        }
    };

    PResult::raise(E::new(I::span(start..end), ErrorKind::Expected(desc)))
}

//------------------------------------------------------------------------------

pub struct TakeExact<Token, Pred, const N: usize> {
    pub(crate) pred: Pred,
    pub(crate) phantom: PhantomData<Token>,
}

impl<Token, Pred, const N: usize> Describe for TakeExact<Token, Pred, N>
where
    Pred: Predicate<Token>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} tokens matches ", N)?;
        self.pred.describe(f)
    }
}

impl<'src, I, Ext, Pred, const N: usize> Pattern<'src, I, Ext> for TakeExact<I::Token, Pred, N>
where
    I: InputSlice<'src> + InputOwnableToken<'src>,
    Ext: Extra<'src, I>,
    Pred: Predicate<I::Token>,
{
    type View<'tmp>
        = [I::Token; N]
    where
        'src: 'tmp;

    #[inline]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &N, input, start.clone()).raise_or_map(|end| {
            let mut len = 0;
            let mut out = MaybeUninit::<[I::Token; N]>::uninit();
            for (i, item) in input.iter_owned(start.clone()..end.clone()).enumerate() {
                unsafe { (&raw mut (*out.as_mut_ptr())[i]).write(item) }
                len = i;
            }

            if len != N {
                panic!("contract violation")
            }

            input.release_slice(start..end.clone());

            (unsafe { out.assume_init() }, end)
        })
    }

    #[inline]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &N, input, start)
    }
}

//------------------------------------------------------------------------------

pub struct SliceExact<Token, Pred, const N: usize> {
    pub(crate) pred: Pred,
    pub(crate) phantom: PhantomData<Token>,
}

impl<Token, Pred, const N: usize> Describe for SliceExact<Token, Pred, N>
where
    Pred: Predicate<Token>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} tokens matches ", N)?;
        self.pred.describe(f)
    }
}

impl<'src, I, Ext, Pred, const N: usize> Pattern<'src, I, Ext> for SliceExact<I::Token, Pred, N>
where
    I: InputSlice<'src> + InputBorrowableToken<'src>,
    Ext: Extra<'src, I>,
    Pred: Predicate<I::Token>,
{
    type View<'tmp>
        = [&'tmp I::Token; N]
    where
        'src: 'tmp;

    #[inline]
    fn __parse<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &N, input, start.clone()).raise_or_map(|end| {
            let mut len = 0;
            let mut out = MaybeUninit::<[&'tmp I::Token; N]>::uninit();
            for (i, item) in input.iter_borrowed(start.clone()..end.clone()).enumerate() {
                unsafe { (&raw mut (*out.as_mut_ptr())[i]).write(item) }
                len = i;
            }

            if len != N {
                panic!("contract violation")
            }

            input.release_slice(start..end.clone());

            (unsafe { out.assume_init() }, end)
        })
    }

    #[inline]
    fn __check<'tmp>(
        &self,
        input: &'tmp mut I,
        start: I::Cursor,
        _state: MaybeMut<Ext::State>,
        _ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<I::Cursor, Ext::Error>
    where
        'src: 'tmp,
    {
        __check_take(self, &self.pred, &N, input, start)
    }
}
