use core::{num::NonZeroUsize, ops::Range};
use memchr::memchr_iter;

pub fn line_col(source: &str, offset: usize) -> Option<(NonZeroUsize, NonZeroUsize)> {
    if offset > source.len() {
        return None;
    }

    let (line, rest) = match memchr_iter(b'\n', source.as_bytes())
        .enumerate()
        .take_while(|(_, off)| *off < offset)
        .last()
    {
        None => (0, source.get(..offset)?),
        Some((line, off)) => (line + 1, source.get(off + 1..offset)?),
    };

    #[cfg(feature = "unicode-segmentation")]
    let col = unicode_segmentation::UnicodeSegmentation::graphemes(rest, true).count();
    #[cfg(not(feature = "unicode-segmentation"))]
    let col = rest.chars().count();

    Some((
        line.saturating_add(1).try_into().unwrap(),
        col.saturating_add(1).try_into().unwrap(),
    ))
}

pub fn line_col_span(
    source: &str,
    range: Range<usize>,
) -> Option<((NonZeroUsize, NonZeroUsize), (NonZeroUsize, NonZeroUsize))> {
    let (before, after) = source.split_at_checked(range.start)?;

    let loc1 = line_col(before, before.len())?;
    let loc2 = line_col(after, range.end - range.start)?;

    Some((loc1, series_locate(loc1, loc2)))
}

pub(crate) fn series_locate(
    loc1: (NonZeroUsize, NonZeroUsize),
    loc2: (NonZeroUsize, NonZeroUsize),
) -> (NonZeroUsize, NonZeroUsize) {
    let (line, col) = loc1;
    let (line2, col2) = loc2;

    if line == line2 {
        (line, col.saturating_add(usize::from(col2) - 1))
    } else {
        (line.saturating_add(usize::from(line2) - 1), col2)
    }
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn nzu(n: usize) -> NonZeroUsize {
        n.try_into().unwrap()
    }

    #[test]
    #[rustfmt::skip]
    fn main() {
        const CONTENT: &str = concat!(
            /* 00..=03 */ "123\n",
            /* 04..=07 */ "ABC\n",
            /* 08..=14 */ "测试\n",
            /* 15..=21 */ "你好\n",
        );
        assert_eq!( line_col(CONTENT,  0), Some((nzu(1), nzu(1))) );
        assert_eq!( line_col(CONTENT,  1), Some((nzu(1), nzu(2))) );
        assert_eq!( line_col(CONTENT,  2), Some((nzu(1), nzu(3))) );
        assert_eq!( line_col(CONTENT,  3), Some((nzu(1), nzu(4))) );
        assert_eq!( line_col(CONTENT,  4), Some((nzu(2), nzu(1))) );
        assert_eq!( line_col(CONTENT,  5), Some((nzu(2), nzu(2))) );
        assert_eq!( line_col(CONTENT,  6), Some((nzu(2), nzu(3))) );
        assert_eq!( line_col(CONTENT,  7), Some((nzu(2), nzu(4))) );
        assert_eq!( line_col(CONTENT,  8), Some((nzu(3), nzu(1))) );
        assert_eq!( line_col(CONTENT,  9), None                   );
        assert_eq!( line_col(CONTENT, 10), None                   );
        assert_eq!( line_col(CONTENT, 11), Some((nzu(3), nzu(2))) );
        assert_eq!( line_col(CONTENT, 12), None                   );
        assert_eq!( line_col(CONTENT, 13), None                   );
        assert_eq!( line_col(CONTENT, 14), Some((nzu(3), nzu(3))) );
        assert_eq!( line_col(CONTENT, 15), Some((nzu(4), nzu(1))) );
        assert_eq!( line_col(CONTENT, 16), None                   );
        assert_eq!( line_col(CONTENT, 17), None                   );
        assert_eq!( line_col(CONTENT, 18), Some((nzu(4), nzu(2))) );
        assert_eq!( line_col(CONTENT, 19), None                   );
        assert_eq!( line_col(CONTENT, 20), None                   );
        assert_eq!( line_col(CONTENT, 21), Some((nzu(4), nzu(3))) );
        assert_eq!( line_col(CONTENT, 22), Some((nzu(5), nzu(1))) );
        assert_eq!( line_col(CONTENT, 23), None                   );

        assert_eq!( line_col_span(CONTENT,  8..8 +  3), Some(((nzu(3), nzu(1)), (nzu(3), nzu(2)))) );
        assert_eq!( line_col_span(CONTENT,  8..8 +  6), Some(((nzu(3), nzu(1)), (nzu(3), nzu(3)))) );
        assert_eq!( line_col_span(CONTENT,  8..8 +  7), Some(((nzu(3), nzu(1)), (nzu(4), nzu(1)))) );
        assert_eq!( line_col_span(CONTENT,  8..8 + 10), Some(((nzu(3), nzu(1)), (nzu(4), nzu(2)))) );
    }
}
