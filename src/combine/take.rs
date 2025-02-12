use super::*;
use core::ops::RangeFrom;

// take_while, e.g. `is_whitespace..`

// impl<'i, P: Predicate<char>> Proceed<'i, str> for RangeFrom<P> {
//     type Capture = &'i str;
//     type State = ();
//
//     fn proceed(&self, slice: &'i str, entry: &mut Self::State, eof: bool) -> ProceedResult {
//         todo!()
//     }
//
//     fn extract(&self, slice: &'i str, entry: Self::State) -> Self::Capture {
//         todo!()
//     }
// }
//
// impl<'i, T: 'i, P: Predicate<T>> Proceed<'i, [T]> for RangeFrom<P> {
//     type Capture = &'i [T];
//     type State = ();
//
//     fn proceed(&self, slice: &'i [T], entry: &mut Self::State, eof: bool) -> ProceedResult {
//         todo!()
//     }
//
//     fn extract(&self, slice: &'i [T], entry: Self::State) -> Self::Capture {
//         todo!()
//     }
// }
