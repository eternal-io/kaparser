use super::*;

#[inline(always)]
pub const fn winged<U, const SINGLE: bool>(primary: U::Item, secondary: U::Item) -> Winged<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        start_primary: primary,
        start_secondary: secondary,
        end_secondary: secondary,
        end_primary: primary,
    }
}
#[inline(always)]
pub const fn winged2<U, const SINGLE: bool>(
    primary: U::Item,
    start_secondary: U::Item,
    end_secondary: U::Item,
) -> Winged<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        start_primary: primary,
        start_secondary,
        end_secondary,
        end_primary: primary,
    }
}
#[inline(always)]
pub const fn winged3<U, const SINGLE: bool>(
    start_primary: U::Item,
    start_secondary: U::Item,
    end_secondary: U::Item,
    end_primary: U::Item,
) -> Winged<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        start_primary,
        start_secondary,
        end_secondary,
        end_primary,
    }
}

#[inline(always)]
pub const fn winged_flipped<U, const SINGLE: bool>(outer: U::Item, inner: U::Item) -> WingedFlipped<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        start_outer: outer,
        start_inner: inner,
        end_inner: inner,
        end_outer: outer,
    }
}
#[inline(always)]
pub const fn winged_flipped2<U, const SINGLE: bool>(
    start_outer: U::Item,
    inner: U::Item,
    end_outer: U::Item,
) -> WingedFlipped<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        start_outer,
        start_inner: inner,
        end_inner: inner,
        end_outer,
    }
}
#[inline(always)]
pub const fn winged_flipped3<U, const SINGLE: bool>(
    start_outer: U::Item,
    start_inner: U::Item,
    end_inner: U::Item,
    end_outer: U::Item,
) -> WingedFlipped<U, SINGLE>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        start_outer,
        start_inner,
        end_inner,
        end_outer,
    }
}

//------------------------------------------------------------------------------

pub struct Winged<U, const SINGLE: bool>
where
    U: ?Sized + ThinSlice,
{
    start_primary: U::Item,
    start_secondary: U::Item,
    end_secondary: U::Item,
    end_primary: U::Item,
}

impl<'i, U, E, const SINGLE: bool> Pattern<'i, U, E> for Winged<U, SINGLE>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = (usize, &'i U);
    type Internal = (usize, usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 0, 0)
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        // Use `winged3('<', '{', '}', '>')` as example, it accepts input like `<<<{ ... }>>>`.
        let (n_primaries, winged_off, winged_content_off) = entry;

        // Matching `<`.
        if *winged_off == 0 {
            // Skipping `<` `ctr` times, and the subsequent `item` may be `{`.
            let Some((ctr, (delim_delta, item))) = slice
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.start_primary)
                .next()
            else {
                // This slice consists only of repeated `<`.
                return match eof {
                    true => E::raise_reject_at(slice.len()),
                    false => E::raise_unfulfilled(None),
                };
            };

            if ctr == 0 && !SINGLE {
                // `!SINGLE` expected at least one `<`.
                return E::raise_reject_at(0);
            }

            if item != self.start_secondary {
                // The subsequent `item` is not `{`.
                return E::raise_reject_at(delim_delta);
            }

            *n_primaries = ctr;
            *winged_off = delim_delta + U::len_of(item); // the offset of `<<<{`
            *winged_content_off = *winged_off;
        }

        loop {
            // Looking for the (offset - winged_content_off) of `}`.
            let Some(content_delta) = slice.split_at(*winged_content_off).1.memchr(self.end_secondary) else {
                return match eof {
                    true => E::raise_halt_at(slice.len()),
                    false => {
                        *winged_content_off = slice.len();
                        E::raise_unfulfilled(None)
                    }
                };
            };

            *winged_content_off += content_delta; // the offset of `<<<{ ... `

            // Total offset of the consumed input, currently only `<<<{ ... }`.
            let mut offset = *winged_content_off + U::len_of(self.end_secondary);

            // Taking `>` at most n_primaries (= ctr + 1) times.
            let m_primaries = match slice
                .split_at(offset)
                .1
                .iter_indices()
                .enumerate()
                .take(*n_primaries)
                .take_while(|(_ctr, (_off, item))| *item == self.end_primary)
                .last()
            {
                None => 0,
                Some((ctr, (delim_delta, _))) => {
                    offset += delim_delta + U::len_of(self.end_primary); // the offset of `<<<{ ... }>>>`, or possibly `<<<{ ... }>` etc.
                    ctr + 1
                }
            };

            if *n_primaries == m_primaries {
                // Closed properly.
                return Ok(offset);
            }

            match offset == slice.len() {
                true => {
                    return match eof {
                        true => E::raise_halt_at(slice.len()), // NOT closed properly.
                        false => E::raise_unfulfilled(None),   // Unable to determine if it was closed properly.
                    };
                }
                false => *winged_content_off = offset, // Fake end sequence, actually part of content.
            }
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (n_primaries, winged_off, winged_content_off) = entry;

        (n_primaries, slice.split_at(winged_content_off).0.split_at(winged_off).1)
    }
}

//------------------------------------------------------------------------------

pub struct WingedFlipped<U, const SINGLE: bool>
where
    U: ?Sized + ThinSlice,
{
    start_outer: U::Item,
    start_inner: U::Item,
    end_inner: U::Item,
    end_outer: U::Item,
}

impl<'i, U, E, const SINGLE: bool> Pattern<'i, U, E> for WingedFlipped<U, SINGLE>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = (usize, &'i U);
    type Internal = (usize, usize, usize);

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        (0, 0, 0)
    }
    #[inline(always)]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        let (n_inners, winged_off, winged_content_off) = entry;

        if *winged_off == 0 {
            let Some(item) = slice.first() else {
                return E::raise_reject_at(0);
            };
            if item != self.start_outer {
                return E::raise_reject_at(0);
            }

            let first_off = U::len_of(self.start_outer);
            let Some((ctr, (delim_delta, _))) = slice
                .split_at(first_off)
                .1
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.start_inner)
                .next()
            else {
                return match eof {
                    true => E::raise_reject_at(first_off),
                    false => E::raise_unfulfilled(None),
                };
            };

            *n_inners = ctr;
            *winged_off = first_off + delim_delta;
            *winged_content_off = *winged_off;
        }

        loop {
            let Some(content_delim_delta) = slice.split_at(*winged_content_off).1.memchr(self.end_outer) else {
                return match eof {
                    true => E::raise_halt_at(slice.len()),
                    false => {
                        *winged_content_off = slice.len().saturating_sub(*n_inners);
                        E::raise_unfulfilled(None)
                    }
                };
            };

            let offset = *winged_content_off + content_delim_delta + U::len_of(self.end_outer);

            if let Some((ctr, (real_winged_content_off, _))) = slice
                .split_at(offset - U::len_of(self.end_outer))
                .0
                .iter_indices()
                .rev()
                .enumerate()
                .take(*n_inners)
                .take_while(|(_ctr, (_off, item))| *item == self.end_inner)
                .last()
            {
                if *n_inners == ctr + 1 {
                    *winged_content_off = real_winged_content_off;
                    return Ok(offset);
                }
            }

            *winged_content_off = offset
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (n_inners, winged_off, winged_content_off) = entry;

        (n_inners, slice.split_at(winged_content_off).0.split_at(winged_off).1)
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::*;

    #[test]
    fn test_winged() {
        test_full_match(
            winged3::<str, true>('＜', '｛', '｝', '＞'),
            vec![
                ("｛ＫＡＥ｝", Ok((0, "ＫＡＥ"))),
                ("＜｛ＫＡＥ｝＞", Ok((1, "ＫＡＥ"))),
                ("＜＜｛ＫＡＥ｝＞＞", Ok((2, "ＫＡＥ"))),
                ("＜＜｛ＫＡＥ｝｝＞＞", Ok((2, "ＫＡＥ｝"))),
                ("＜＜｛ＫＡＥ｝＞｝＞＞", Ok((2, "ＫＡＥ｝＞"))),
                ("＜＜｛＜｛ＫＡＥ｝＞｝＞＞", Ok((2, "＜｛ＫＡＥ｝＞"))),
                ("｛ＫＡＥ", Err(4 * 3)),
                ("＜｛ＫＡＥ｝", Err(6 * 3)),
                ("＜＜｛ＫＡＥ｝＞", Err(8 * 3)),
                ("＜｛ＫＡＥ｝＞｝＞", Err(7 * 3)),
            ],
        );
        test_partial_match(
            winged3::<str, true>('＜', '｛', '｝', '＞'),
            vec![
                ("｛ＫＡＥ｝＞", Ok(((0, "ＫＡＥ"), "＞"))),
                ("＜｛ＫＡＥ｝＞｝＞", Ok(((1, "ＫＡＥ"), "｝＞"))),
                ("＜＜｛ＫＡＥ｝＞＞｝＃", Ok(((2, "ＫＡＥ"), "｝＃"))),
            ],
        );
    }

    #[test]
    fn test_winged_flipped() {
        test_full_match(winged_flipped3::<str, true>('｛', '＜', '＞', '｝'), vec![]);
    }
}
