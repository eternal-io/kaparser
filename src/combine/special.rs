use super::*;

#[inline]
pub const fn winged<U, const STRICT: bool>(primary: U::Item, secondary: U::Item) -> Winged<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        primary_start: primary,
        secondary_start: secondary,
        secondary_end: secondary,
        primary_end: primary,
    }
}
#[inline]
pub const fn winged2<U, const STRICT: bool>(
    primary: U::Item,
    secondary_start: U::Item,
    secondary_end: U::Item,
) -> Winged<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        primary_start: primary,
        secondary_start,
        secondary_end,
        primary_end: primary,
    }
}
#[inline]
pub const fn winged3<U, const STRICT: bool>(
    primary_start: U::Item,
    secondary_start: U::Item,
    secondary_end: U::Item,
    primary_end: U::Item,
) -> Winged<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    Winged {
        primary_start,
        secondary_start,
        secondary_end,
        primary_end,
    }
}

#[inline]
pub const fn winged_flipped<U, const STRICT: bool>(outer: U::Item, inner: U::Item) -> WingedFlipped<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        outer_start: outer,
        inner_start: inner,
        inner_end: inner,
        outer_end: outer,
    }
}
#[inline]
pub const fn winged2_flipped<U, const STRICT: bool>(
    outer_start: U::Item,
    inner: U::Item,
    outer_end: U::Item,
) -> WingedFlipped<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        outer_start,
        inner_start: inner,
        inner_end: inner,
        outer_end,
    }
}
#[inline]
pub const fn winged3_flipped<U, const STRICT: bool>(
    outer_start: U::Item,
    inner_start: U::Item,
    inner_end: U::Item,
    outer_end: U::Item,
) -> WingedFlipped<U, STRICT>
where
    U: ?Sized + ThinSlice,
{
    WingedFlipped {
        outer_start,
        inner_start,
        inner_end,
        outer_end,
    }
}

//------------------------------------------------------------------------------

pub struct Winged<U, const STRICT: bool>
where
    U: ?Sized + ThinSlice,
{
    primary_start: U::Item,
    secondary_start: U::Item,
    secondary_end: U::Item,
    primary_end: U::Item,
}

impl<'i, U, E, const STRICT: bool> Pattern<'i, U, E> for Winged<U, STRICT>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = (usize, &'i U);
    type Internal = (usize, usize, usize);

    #[inline]
    fn init(&self) -> Self::Internal {
        (0, 0, 0)
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        // Use `winged3('<', '{', '}', '>')` as example, it accepts input like `<<<{ ... }>>>`.
        let (n_primaries, winged_off, winged_content_off) = entry;

        if *winged_off == 0 {
            // Skipping `<` `ctr` times, and the subsequent `item` may be `{`.
            let Some((ctr, (delim_delta, item))) = slice
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.primary_start)
                .next()
            else {
                // This slice consists only of repeated `<`.
                return match eof {
                    true => E::raise_reject_at(slice.len()),
                    false => E::raise_unfulfilled(None),
                };
            };

            if ctr == 0 && STRICT {
                // `STRICT` expected at least one `<`.
                return E::raise_reject_at(0);
            }

            if item != self.secondary_start {
                // The subsequent `item` is not `{`.
                return E::raise_reject_at(delim_delta);
            }

            *n_primaries = ctr;
            *winged_off = delim_delta + U::len_of(item); // the offset of `<<<{`
            *winged_content_off = *winged_off;
        }

        loop {
            // Looking for the (offset - winged_content_off) of `}`.
            let Some((content_delta, _)) = slice.after(*winged_content_off).memchr(&[self.secondary_end]) else {
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
            let mut offset = *winged_content_off + U::len_of(self.secondary_end);

            // Taking `>` at most n_primaries (= ctr + 1) times.
            let m_primaries = match slice
                .after(offset)
                .iter_indices()
                .enumerate()
                .take(*n_primaries)
                .take_while(|(_ctr, (_off, item))| *item == self.primary_end)
                .last()
            {
                None => 0,
                Some((ctr, (delim_delta, _))) => {
                    offset += delim_delta + U::len_of(self.primary_end); // the offset of `<<<{ ... }>>>`, or possibly `<<<{ ... }>` etc.
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
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (n_primaries, winged_off, winged_content_off) = entry;

        (n_primaries, slice.subslice(winged_off..winged_content_off))
    }
}

//------------------------------------------------------------------------------

pub struct WingedFlipped<U, const STRICT: bool>
where
    U: ?Sized + ThinSlice,
{
    outer_start: U::Item,
    inner_start: U::Item,
    inner_end: U::Item,
    outer_end: U::Item,
}

impl<'i, U, E, const STRICT: bool> Pattern<'i, U, E> for WingedFlipped<U, STRICT>
where
    U: ?Sized + ThinSlice + 'i,
    E: Situation,
{
    type Captured = (usize, &'i U);
    type Internal = (usize, usize, usize);

    #[inline]
    fn init(&self) -> Self::Internal {
        (0, 0, 0)
    }
    #[inline]
    fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
        // Use `winged_flipped3('{', '<', '>', '}')` as example, it accepts input like `{<<< ... >>>}`.
        let (n_inners, winged_off, winged_content_off) = entry;

        if *winged_off == 0 {
            // Matching the leading `{`.
            let Some(item) = slice.first() else {
                return E::raise_reject_at(0);
            };
            if item != self.outer_start {
                return E::raise_reject_at(0);
            }

            let first_off = U::len_of(self.outer_start); // the offset of `{`

            // Skipping `<` `ctr` times.
            let Some((ctr, (delim_delta, _))) = slice
                .after(first_off)
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.inner_start)
                .next()
            else {
                // The item other than `<` is not encountered, the start sequence may not be completed yet.
                return match eof {
                    true => E::raise_reject_at(slice.len()),
                    false => E::raise_unfulfilled(None),
                };
            };

            if ctr == 0 && STRICT {
                // `STRICT` expected at least one `<`.
                return E::raise_reject_at(0);
            }

            *n_inners = ctr;
            *winged_off = first_off + delim_delta; // the offset of `{<<<`
            *winged_content_off = *winged_off;
        }

        loop {
            // Looking for the (offset - winged_content_off) of `}`.
            let Some((content_delim_delta, _)) = slice.after(*winged_content_off).memchr(&[self.outer_end]) else {
                return match eof {
                    true => E::raise_halt_at(slice.len()),
                    false => {
                        // Step conservatively to ensure the end sequence is not missed.
                        *winged_content_off = slice.len().saturating_sub(*n_inners * U::len_of(self.inner_end));
                        E::raise_unfulfilled(None)
                    }
                };
            };

            let winged_content_winner_off = *winged_content_off + content_delim_delta; // the offset of `{<<< ... ???`

            // Total offset of the consumed input, currently is `{<<< ... ???}`.
            let offset = winged_content_winner_off + U::len_of(self.outer_end);

            // Taking `>` (`???`) at most n_inners (= ctr + 1) times in reversed order.
            let (m_inners, real_winged_content_off) = match slice
                .before(winged_content_winner_off)
                .iter_indices()
                .rev()
                .enumerate()
                .take(*n_inners)
                .take_while(|(_ctr, (_off, item))| *item == self.inner_end)
                .last()
            {
                None => (0, winged_content_winner_off),
                Some((ctr, (winged_content_off, _))) => (ctr + 1, winged_content_off),
            };

            if *n_inners == m_inners {
                // Closed properly.
                *winged_content_off = real_winged_content_off;
                return Ok(offset);
            }

            // Fake end sequence (because not enough `>`), actually part of content.
            *winged_content_off = offset;
        }
    }
    #[inline]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let (n_inners, winged_off, winged_content_off) = entry;

        (n_inners, slice.subslice(winged_off..winged_content_off))
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winged() {
        test_full_match(
            winged3::<str, true>('＜', '｛', '｝', '＞'),
            vec![
                ("＜｛ＫＡＥ｝＞", Ok((1, "ＫＡＥ"))),
                ("＜＜｛ＫＡＥ｝＞＞", Ok((2, "ＫＡＥ"))),
                ("＜＜｛ＫＡＥ｝｝＞＞", Ok((2, "ＫＡＥ｝"))),
                ("＜＜｛ＫＡＥ｝＞｝＞＞", Ok((2, "ＫＡＥ｝＞"))),
                ("＜＜｛＜｛ＫＡＥ｝＞｝＞＞", Ok((2, "＜｛ＫＡＥ｝＞"))),
                ("｛ＫＡＥ", Err(0 * 3)),
                ("｛ＫＡＥ｝", Err(0 * 3)),
                ("ＫＡＥ｝", Err(0 * 3)),
                ("＜ＫＡＥ｝", Err(1 * 3)),
                ("＜ＫＡＥ＞", Err(1 * 3)),
                ("＜｛ＫＡＥ｝", Err(6 * 3)),
                ("＜＜｛ＫＡＥ｝＞", Err(8 * 3)),
                ("＜｛ＫＡＥ｝＞｝＞", Err(7 * 3)),
            ],
        );
        test_full_match(
            winged3::<str, false>('＜', '｛', '｝', '＞'),
            vec![
                ("｛ＫＡＥ", Err(4 * 3)),
                ("｛ＫＡＥ｝", Ok((0, "ＫＡＥ"))),
                ("｛ＫＡＥ＞｝", Ok((0, "ＫＡＥ＞"))),
            ],
        );
        test_partial_match(
            winged3::<str, true>('＜', '｛', '｝', '＞'),
            vec![
                ("｛ＫＡＥ｝＞", Err(0 * 3)),
                ("＜｛ＫＡＥ｝＞｝＞", Ok(((1, "ＫＡＥ"), "｝＞"))),
                ("＜＜｛ＫＡＥ｝＞＞｝＃", Ok(((2, "ＫＡＥ"), "｝＃"))),
            ],
        );
        test_partial_match(
            winged3::<str, false>('＜', '｛', '｝', '＞'),
            vec![("｛ＫＡＥ｝＞", Ok(((0, "ＫＡＥ"), "＞")))],
        );
    }

    #[test]
    fn test_winged_flipped() {
        test_full_match(
            winged3_flipped::<str, true>('｛', '＜', '＞', '｝'),
            vec![
                ("｛＜ＫＡＥ＞｝", Ok((1, "ＫＡＥ"))),
                ("｛＜＜ＫＡＥ＞＞｝", Ok((2, "ＫＡＥ"))),
                ("｛＜＜ＫＡＥ＞＞＞｝", Ok((2, "ＫＡＥ＞"))),
                ("｛＜＜ＫＡＥ＞｝＞＞｝", Ok((2, "ＫＡＥ＞｝"))),
                ("｛＜＜ＫＡＥ＞＞＞＞｝", Ok((2, "ＫＡＥ＞＞"))),
                ("｛＜＜＜ＫＡＥ＞＞＞｝", Ok((3, "ＫＡＥ"))),
                ("｛＜ＫＡＥ＞＞｝", Ok((1, "ＫＡＥ＞"))),
                ("｛ＫＡＥ｝", Err(0 * 3)),
                ("｛ＫＡＥ＞｝", Err(0 * 3)),
                ("ＫＡＥ｝", Err(0 * 3)),
                ("｛ＫＡＥ｝＞", Err(0 * 3)),
                ("｛＜ＫＡＥ｝", Err(6 * 3)),
                ("｛＜＜ＫＡＥ｝", Err(7 * 3)),
                ("｛＜＜ＫＡＥ＞｝", Err(8 * 3)),
            ],
        );
        test_full_match(
            winged3_flipped::<str, false>('｛', '＜', '＞', '｝'),
            vec![
                ("｛ＫＡＥ｝＞", Err(5 * 3)),
                ("｛ＫＡＥ｝", Ok((0, "ＫＡＥ"))),
                ("｛ＫＡＥ＞｝", Ok((0, "ＫＡＥ＞"))),
            ],
        );
        test_partial_match(
            winged3_flipped::<str, true>('｛', '＜', '＞', '｝'),
            vec![
                ("｛ＫＡＥ｝＞", Err(0 * 3)),
                ("｛＜ＫＡＥ＞｝＞", Ok(((1, "ＫＡＥ"), "＞"))),
                ("｛＜＜ＫＡＥ＞＞｝＞＃", Ok(((2, "ＫＡＥ"), "＞＃"))),
            ],
        );
        test_partial_match(
            winged3_flipped::<str, false>('｛', '＜', '＞', '｝'),
            vec![("｛ＫＡＥ｝＞", Ok(((0, "ＫＡＥ"), "＞")))],
        );
    }
}
