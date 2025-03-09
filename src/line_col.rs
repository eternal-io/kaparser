use core::num::NonZeroUsize;
use memchr::memchr_iter;

pub fn line_col(slice: &str, offset: usize) -> Option<(NonZeroUsize, NonZeroUsize)> {
    if offset > slice.len() {
        return None;
    }

    let (line, rest) = match memchr_iter(b'\n', slice.as_bytes())
        .enumerate()
        .take_while(|(_, off)| *off < offset)
        .last()
    {
        None => (1usize.try_into().unwrap(), slice.get(..offset)?),
        Some((line, off)) => (line.saturating_add(1).try_into().unwrap(), slice.get(..off)?),
    };

    Some((line, rest.chars().count().saturating_add(1).try_into().unwrap()))
}

pub fn line_col_span(
    slice: &str,
    offset: usize,
    length: usize,
) -> Option<((NonZeroUsize, NonZeroUsize), (NonZeroUsize, NonZeroUsize))> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = concat!(
        /* 00..10 */ "123456789\n",
        /* 10..17 */ "ABCDEF\n",
        /* 17..24 */ "测试\n",
        /* 24..30 */ "你好",
    );

    fn nzu(n: usize) -> NonZeroUsize {
        n.try_into().unwrap()
    }

    #[test]
    fn main() {
        assert_eq!(line_col(CONTENT, 0), Some((nzu(1), nzu(1))));
        assert_eq!(line_col(CONTENT, 8), Some((nzu(1), nzu(9))));
        assert_eq!(line_col(CONTENT, 9), Some((nzu(1), nzu(10))));
        assert_eq!(line_col(CONTENT, 10), Some((nzu(1), nzu(10))));
        assert_eq!(line_col(CONTENT, 11), Some((nzu(2), nzu(1))));
        assert_eq!(line_col(CONTENT, 12), Some((nzu(2), nzu(1))));
    }
}
