use crate::{common::*, error::*, extra::*, input::*, pattern::*, predicate::*, private, slice::*};
use core::{fmt, marker::PhantomData};

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
                false => Err(self[0].report(I::span(start..end))),
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
                false => Err(self.0.report(I::span(start..end))),
            })
    }
}

//------------------------------------------------------------------------------

pub struct Take<Token, Pred, R> {
    pub(crate) pred: Pred,
    pub(crate) range: R,
    pub(crate) phantom: PhantomData<Token>,
}

impl<Token, Pred, R> Describe for &Take<Token, Pred, R>
where
    Pred: Predicate<Token>,
    R: URangeBounds,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
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
        state: MaybeMut<Ext::State>,
        ctx: MaybeRef<Ext::Context>,
        _: private::Token,
    ) -> PResult<(Self::View<'tmp>, I::Cursor), Ext::Error>
    where
        'src: 'tmp,
    {
        Pattern::<I, Ext>::__check(self, input, start.clone(), state, ctx, private::Token)
            .raise_or_map(|end| (input.discard_slice(start..end.clone()), end))
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
        let mut times = 0;
        let mut offset = 0;
        loop {
            let (slice, eof) = match input.fetch_slice(start.clone()) {
                Ok(val) => val,
                Err(e) => return PResult::raise(e),
            };

            if let Some((i, (off, item))) = slice
                .after(offset)
                .iter_indices()
                .enumerate()
                .take_while(|(i, (_off, item))| self.range.unfulfilled(times + *i) && item.verify_by(&self.pred))
                .last()
            {
                times += i + 1;
                offset += off + slice.len_of(item.as_ref());
            }

            let end = I::bump_cursor(start.clone(), offset);

            if !eof && self.range.unfulfilled(times) {
                continue;
            } else if self.range.contains(times) {
                return PResult::emit(end);
            } else {
                return PResult::raise(Ext::Error::new(I::span(start..end), ErrorKind::Expected(&self)));
            }
        }
    }
}
