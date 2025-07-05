use crate::{common::*, extra::*, input::*, pattern::*, predicate::*, private};

impl<'src, I, Ext, P> Pattern<'src, I, Ext> for [P; 1]
where
    I: InputOwnableToken<'src>,
    Ext: Extra<'src, I>,
    P: Predicate<I::Token>,
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

        match input.next_maybe_ref::<Ext::Error>(&mut end) {
            Err(e) => PResult::raise(e),
            Ok(opt) => {
                if let Some(token) = opt {
                    if token.verify_by(&self[0]) {
                        return PResult::submit(end);
                    }
                }

                self[0].raise(I::span(start..end))
            }
        }
    }
}
