/// You can abbreviate `n..=n` to `n`.
pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn want_more(&self, times: usize) -> bool;
}

#[rustfmt::skip]
mod urange_bounds {
    use super::URangeBounds;
    use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

    impl URangeBounds for usize {
        fn contains(&self, times: usize) -> bool { times == *self }
        fn want_more(&self, times: usize) -> bool { times < *self }
    }
    impl URangeBounds for RangeFull {
        fn contains(&self, _t: usize) -> bool { true }
        fn want_more(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for RangeFrom<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for Range<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times < *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn want_more(&self, times: usize) -> bool { times < self.end }
    }
}
