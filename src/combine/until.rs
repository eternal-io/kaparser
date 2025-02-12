use super::*;
use core::ops::RangeTo;

// skim_until, e.g. `..is_newline`, `.."*/"`
//
// impl<'i, P: Predicate<char>> Proceed<'i, str> for RangeTo<P> {
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
// impl<'i, T: 'i, P: Proceed<'i, [T]>> Proceed<'i, [T]> for RangeTo<P> {
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
