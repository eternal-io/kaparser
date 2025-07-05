use crate::{common::*, extra::*, input::*, pattern::*, predicate::*, private};

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
