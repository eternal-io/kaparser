use super::*;

#[inline(always)]
pub const fn winged<U>(primary: U::Item, secondary: U::Item) -> Winged<U>
where
    U: ?Sized + ThinSlice,
{
    Winged { primary, secondary }
}

#[inline(always)]
pub const fn winged_flipped<U>(outer: U::Item, inner: U::Item)
where
    U: ?Sized + ThinSlice,
{
}

//------------------------------------------------------------------------------

pub struct Winged<U>
where
    U: ?Sized + ThinSlice,
{
    primary: U::Item,
    secondary: U::Item,
}

impl<'i, U, E> Pattern<'i, U, E> for Winged<U>
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
            let Some((ctr, (head_delta, item))) = slice
                .iter_indices()
                .enumerate()
                .skip_while(|(_ctr, (_off, item))| *item == self.primary)
                .next()
            else {
                return E::raise_reject_at(slice.len());
            };

            if item != self.secondary {
                return E::raise_reject_at(head_delta);
            }

            *n_primaries = ctr;
            *winged_off = head_delta + U::len_of(item);
            *winged_content_off = *winged_off;
        }

        loop {
            let Some(content_delta) = slice.split_at(*winged_content_off).1.memchr(self.secondary) else {
                return match eof {
                    true => E::raise_halt_at(slice.len()),
                    false => {
                        *winged_content_off = slice.len();
                        E::raise_unfulfilled(None)
                    }
                };
            };

            *winged_content_off += content_delta;

            let mut offset = *winged_content_off + U::len_of(self.secondary);

            match slice
                .split_at(offset)
                .1
                .iter_indices()
                .enumerate()
                .take(*n_primaries)
                .take_while(|(_ctr, (_off, item))| *item == self.primary)
                .last()
            {
                Some((ctr, (tail_delta, _))) => {
                    offset += tail_delta + U::len_of(self.primary);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winged() {
        let pat = impls::opaque_simple::<str, _>(winged('#', '"'));

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
}
