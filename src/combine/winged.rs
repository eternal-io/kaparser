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
        let (n_primaries, winged_off, winged_content_off) = entry;

        if *winged_off == 0 {
            let Some((ctr, (delim_delta, item))) = slice
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.start_primary)
                .next()
            else {
                return E::raise_reject_at(slice.len());
            };

            if item != self.start_secondary {
                return E::raise_reject_at(delim_delta);
            }

            *n_primaries = ctr;
            *winged_off = delim_delta + U::len_of(item);
            *winged_content_off = *winged_off;
        }

        loop {
            let Some(content_delta) = slice.split_at(*winged_content_off).1.memchr(self.end_secondary) else {
                return match eof {
                    true => E::raise_halt_at(slice.len()),
                    false => {
                        *winged_content_off = slice.len();
                        E::raise_unfulfilled(None)
                    }
                };
            };

            *winged_content_off += content_delta;

            let mut offset = *winged_content_off + U::len_of(self.end_secondary);

            match slice
                .split_at(offset)
                .1
                .iter_indices()
                .enumerate()
                .take(*n_primaries)
                .take_while(|(_ctr, (_off, item))| *item == self.end_primary)
                .last()
            {
                Some((ctr, (delim_delta, _))) => {
                    offset += delim_delta + U::len_of(self.end_primary);

                    match *n_primaries == ctr + 1 {
                        true => return Ok(offset),
                        false => *winged_content_off = offset,
                    }
                }
                None => {
                    match *n_primaries == 0 {
                        true => return Ok(offset),
                        false => *winged_content_off = offset,
                    };
                }
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

    #[test]
    fn test_winged() {
        let pat = impls::opaque_simple(winged::<str, true>('#', '"'));
        assert_eq!(pat.full_match(r###""...""###).unwrap(), (0, r###"..."###));
        assert_eq!(pat.full_match(r###"#"..."#"###).unwrap(), (1, r###"..."###));
        assert_eq!(pat.full_match(r###"##"..."##"###).unwrap(), (2, r###"..."###));
        assert_eq!(pat.full_match(r###"##"..""##"###).unwrap(), (2, r###"..""###));
        assert_eq!(pat.full_match(r###"##"."#"##"###).unwrap(), (2, r###"."#"###));
        assert_eq!(pat.full_match(r###"##"""#"##"###).unwrap(), (2, r###"""#"###));
        assert_eq!(pat.full_match(r###"##"##"###).unwrap_err().offset(), 5);
        assert_eq!(pat.full_match(r###"###"###).unwrap_err().offset(), 3);
        assert_eq!(pat.full_match(r###"#'...'#"###).unwrap_err().offset(), 1);
        assert_eq!(pat.full_match(r###"..."###).unwrap_err().offset(), 0);
    }
    #[test]
    fn test_winged2() {
        let pat = impls::opaque_simple(winged2::<str, true>('#', '<', '>'));
        assert_eq!(pat.full_match(r###"<...>"###).unwrap(), (0, r###"..."###));
        assert_eq!(pat.full_match(r###"#<...>#"###).unwrap(), (1, r###"..."###));
        assert_eq!(pat.full_match(r###"##<...>##"###).unwrap(), (2, r###"..."###));
        assert_eq!(pat.full_match(r###"##<..>>##"###).unwrap(), (2, r###"..>"###));
        assert_eq!(pat.full_match(r###"##<.>#>##"###).unwrap(), (2, r###".>#"###));
        assert_eq!(pat.full_match(r###"##<###>##"###).unwrap(), (2, r###"###"###));
        assert_eq!(pat.full_match(r###"##<##>"###).unwrap_err().offset(), 6);
        assert_eq!(pat.full_match(r###"#'...'#"###).unwrap_err().offset(), 1);
    }
    #[test]
    fn test_winged3() {
        let pat = impls::opaque_simple(winged3::<str, true>('<', '[', ']', '>'));
        assert_eq!(pat.full_match(r###"[...]"###).unwrap(), (0, r###"..."###));
        assert_eq!(pat.full_match(r###"<[...]>"###).unwrap(), (1, r###"..."###));
        assert_eq!(pat.full_match(r###"<<[]>>"###).unwrap(), (2, r###""###));
        assert_eq!(pat.full_match(r###"<<[.]>>"###).unwrap(), (2, r###"."###));
        assert_eq!(pat.full_match(r###"<<[..]>>"###).unwrap(), (2, r###".."###));
        assert_eq!(pat.full_match(r###"<<[...]>>"###).unwrap(), (2, r###"..."###));
        assert_eq!(pat.full_match(r###"<<<>>>"###).unwrap_err().offset(), 3);
        assert_eq!(pat.full_match(r###"<'...'>"###).unwrap_err().offset(), 1);
    }

    #[test]
    fn test_winged_flipped() {
        let pat = impls::opaque_simple(winged_flipped::<str, false>('/', '*'));
        assert_eq!(pat.full_match("/*...*/").unwrap(), (1, "..."));
        assert_eq!(pat.full_match("/**...**/").unwrap(), (2, "..."));
        assert_eq!(pat.full_match("/***...***/").unwrap(), (3, "..."));
        assert_eq!(pat.full_match("/*...*/*/").unwrap_err().offset(), 7);
        assert_eq!(pat.full_match("/*...*/").unwrap(), (1, "..."));
        assert_eq!(pat.full_match("/*...*").unwrap_err().offset(), 6);
        assert_eq!(pat.full_match("/*...").unwrap_err().offset(), 5);
        assert_eq!(pat.full_match("/...*/").unwrap_err().offset(), 1);
    }

    #[test]
    fn test_winged_flipped2() {
        let pat = impls::opaque_simple(winged_flipped2::<str, false>('(', '*', ')'));
        assert_eq!(pat.full_match("(*...*)").unwrap(), (1, "..."));
        assert_eq!(pat.full_match("((*...*)").unwrap(), (1, "(*..."));
        assert_eq!(pat.full_match("(*...*").unwrap_err().offset(), 6);
        assert_eq!(pat.full_match("(*...").unwrap_err().offset(), 5);
        assert_eq!(pat.full_match("(...*)").unwrap_err().offset(), 1);
    }

    #[test]
    fn test_winged_flipped3() {
        let pat = impls::opaque_simple(winged_flipped3::<str, false>('{', '<', '>', '}'));
        assert_eq!(pat.full_match("{<...>}").unwrap(), (1, "..."));
        assert_eq!(pat.full_match("{{<...>}}").unwrap(), (2, "..."));
        assert_eq!(pat.full_match("{{{<...>}}}").unwrap(), (3, "..."));
        assert_eq!(pat.full_match("{<...>}>}").unwrap_err().offset(), 8);
        assert_eq!(pat.full_match("{<...>").unwrap_err().offset(), 6);
        assert_eq!(pat.full_match("{<...").unwrap_err().offset(), 5);
        assert_eq!(pat.full_match("{...>}").unwrap_err().offset(), 1);
    }
}
