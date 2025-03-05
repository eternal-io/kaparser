#![allow(clippy::type_complexity)]
#![allow(non_snake_case)]
use crate::anything::*;

pub const fn LINE_END<'i, E: Situation>() -> Compound<'i, str, E, (Optional<'i, str, E, &'i str>, &'i str)> {
    com((opt("\r"), "\n"))
}
